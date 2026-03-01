//! Relay protocol messages - must match whisper-relay protocol.

use serde::{Deserialize, Serialize};

/// Message type byte values
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    AuthChallenge = 0x01,
    AuthResponse = 0x02,
    AuthSuccess = 0x03,
    AuthFailure = 0x04,
    Send = 0x10,
    Receive = 0x11,
    Ack = 0x12,
    SendAck = 0x13,
    FetchPending = 0x20,
    PendingCount = 0x21,
    Error = 0xF0,
    Ping = 0xFE,
    Pong = 0xFF,
}

impl TryFrom<u8> for MessageType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x01 => Ok(Self::AuthChallenge),
            0x02 => Ok(Self::AuthResponse),
            0x03 => Ok(Self::AuthSuccess),
            0x04 => Ok(Self::AuthFailure),
            0x10 => Ok(Self::Send),
            0x11 => Ok(Self::Receive),
            0x12 => Ok(Self::Ack),
            0x13 => Ok(Self::SendAck),
            0x20 => Ok(Self::FetchPending),
            0x21 => Ok(Self::PendingCount),
            0xF0 => Ok(Self::Error),
            0xFE => Ok(Self::Ping),
            0xFF => Ok(Self::Pong),
            _ => Err(ProtocolError::InvalidMessageType(value)),
        }
    }
}

/// Send status from relay
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendStatus {
    Queued = 0x00,
    Delivered = 0x01,
    Rejected = 0x02,
}

impl TryFrom<u8> for SendStatus {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x00 => Ok(Self::Queued),
            0x01 => Ok(Self::Delivered),
            0x02 => Ok(Self::Rejected),
            _ => Err(ProtocolError::InvalidStatus(value)),
        }
    }
}

/// Protocol messages
#[derive(Debug, Clone)]
pub enum RelayMessage {
    AuthChallenge { nonce: [u8; 32] },
    AuthResponse { peer_id: [u8; 32], signature: [u8; 64] },
    AuthSuccess,
    AuthFailure { reason: String },
    Send { recipient: [u8; 32], message_id: [u8; 16], payload: Vec<u8> },
    Receive { sender: [u8; 32], message_id: [u8; 16], timestamp: u64, payload: Vec<u8> },
    Ack { message_id: [u8; 16] },
    SendAck { message_id: [u8; 16], status: SendStatus },
    FetchPending,
    PendingCount { count: u32 },
    Error { code: u8, message: String },
    Ping,
    Pong,
}

impl RelayMessage {
    /// Encode message to binary
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::AuthResponse { peer_id, signature } => {
                buf.push(MessageType::AuthResponse as u8);
                buf.extend_from_slice(peer_id);
                buf.extend_from_slice(signature);
            }
            Self::Send { recipient, message_id, payload } => {
                buf.push(MessageType::Send as u8);
                buf.extend_from_slice(recipient);
                buf.extend_from_slice(message_id);
                buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
                buf.extend_from_slice(payload);
            }
            Self::Ack { message_id } => {
                buf.push(MessageType::Ack as u8);
                buf.extend_from_slice(message_id);
            }
            Self::FetchPending => {
                buf.push(MessageType::FetchPending as u8);
            }
            Self::Ping => {
                buf.push(MessageType::Ping as u8);
            }
            Self::Pong => {
                buf.push(MessageType::Pong as u8);
            }
            _ => {} // Server-only messages
        }
        buf
    }

    /// Decode binary message
    pub fn decode(data: &[u8]) -> Result<Self, ProtocolError> {
        if data.is_empty() {
            return Err(ProtocolError::EmptyMessage);
        }

        let msg_type = MessageType::try_from(data[0])?;
        let payload = &data[1..];

        match msg_type {
            MessageType::AuthChallenge => {
                if payload.len() < 32 {
                    return Err(ProtocolError::InvalidLength);
                }
                let mut nonce = [0u8; 32];
                nonce.copy_from_slice(&payload[..32]);
                Ok(Self::AuthChallenge { nonce })
            }
            MessageType::AuthSuccess => Ok(Self::AuthSuccess),
            MessageType::AuthFailure => {
                let reason = String::from_utf8_lossy(payload).to_string();
                Ok(Self::AuthFailure { reason })
            }
            MessageType::Receive => {
                if payload.len() < 32 + 16 + 8 + 4 {
                    return Err(ProtocolError::InvalidLength);
                }
                let mut sender = [0u8; 32];
                sender.copy_from_slice(&payload[..32]);
                let mut message_id = [0u8; 16];
                message_id.copy_from_slice(&payload[32..48]);
                let timestamp = u64::from_be_bytes(payload[48..56].try_into().unwrap());
                let len = u32::from_be_bytes(payload[56..60].try_into().unwrap()) as usize;
                if payload.len() < 60 + len {
                    return Err(ProtocolError::InvalidLength);
                }
                let data = payload[60..60 + len].to_vec();
                Ok(Self::Receive { sender, message_id, timestamp, payload: data })
            }
            MessageType::SendAck => {
                if payload.len() < 17 {
                    return Err(ProtocolError::InvalidLength);
                }
                let mut message_id = [0u8; 16];
                message_id.copy_from_slice(&payload[..16]);
                let status = SendStatus::try_from(payload[16])?;
                Ok(Self::SendAck { message_id, status })
            }
            MessageType::PendingCount => {
                if payload.len() < 4 {
                    return Err(ProtocolError::InvalidLength);
                }
                let count = u32::from_be_bytes(payload[..4].try_into().unwrap());
                Ok(Self::PendingCount { count })
            }
            MessageType::Error => {
                if payload.is_empty() {
                    return Err(ProtocolError::InvalidLength);
                }
                let code = payload[0];
                let message = String::from_utf8_lossy(&payload[1..]).to_string();
                Ok(Self::Error { code, message })
            }
            MessageType::Ping => Ok(Self::Ping),
            MessageType::Pong => Ok(Self::Pong),
            _ => Err(ProtocolError::UnexpectedMessage(msg_type as u8)),
        }
    }
}

/// Protocol errors
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Empty message")]
    EmptyMessage,
    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),
    #[error("Invalid status: {0}")]
    InvalidStatus(u8),
    #[error("Invalid message length")]
    InvalidLength,
    #[error("Unexpected message type: {0}")]
    UnexpectedMessage(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_roundtrip() {
        assert_eq!(MessageType::try_from(0x01).unwrap(), MessageType::AuthChallenge);
        assert_eq!(MessageType::try_from(0x10).unwrap(), MessageType::Send);
        assert!(MessageType::try_from(0x99).is_err());
    }

    #[test]
    fn test_encode_decode_ping() {
        let msg = RelayMessage::Ping;
        let encoded = msg.encode();
        assert_eq!(encoded, vec![0xFE]);
        let decoded = RelayMessage::decode(&encoded).unwrap();
        assert!(matches!(decoded, RelayMessage::Ping));
    }

    #[test]
    fn test_encode_auth_response() {
        let msg = RelayMessage::AuthResponse {
            peer_id: [1u8; 32],
            signature: [2u8; 64],
        };
        let encoded = msg.encode();
        assert_eq!(encoded.len(), 1 + 32 + 64);
        assert_eq!(encoded[0], MessageType::AuthResponse as u8);
    }

    #[test]
    fn test_decode_auth_challenge() {
        let mut data = vec![0x01]; // AuthChallenge
        data.extend_from_slice(&[42u8; 32]);
        let msg = RelayMessage::decode(&data).unwrap();
        match msg {
            RelayMessage::AuthChallenge { nonce } => {
                assert_eq!(nonce, [42u8; 32]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_decode_send_ack() {
        let mut data = vec![0x13]; // SendAck
        data.extend_from_slice(&[1u8; 16]); // message_id
        data.push(0x01); // Delivered
        let msg = RelayMessage::decode(&data).unwrap();
        match msg {
            RelayMessage::SendAck { message_id, status } => {
                assert_eq!(message_id, [1u8; 16]);
                assert_eq!(status, SendStatus::Delivered);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
