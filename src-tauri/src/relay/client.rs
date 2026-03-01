//! WebSocket relay client.

use std::sync::Arc;
use std::time::Duration;

use ed25519_dalek::SigningKey;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

use super::protocol::{ProtocolError, RelayMessage};
use crate::crypto::{keys::peer_id_bytes, signature::sign};

/// Relay client error
#[derive(Debug, thiserror::Error)]
pub enum RelayError {
    #[error("Connection failed: {0}")]
    Connection(String),
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    #[error("Not connected")]
    NotConnected,
    #[error("Protocol error: {0}")]
    Protocol(#[from] ProtocolError),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("Send failed")]
    SendFailed,
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Authenticating,
    Connected,
    Reconnecting,
}

/// Incoming message from relay
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub sender: [u8; 32],
    pub message_id: [u8; 16],
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

/// Message handler callback
pub type MessageHandler = Arc<dyn Fn(IncomingMessage) + Send + Sync>;

/// Relay client for whisper messaging
pub struct RelayClient {
    url: String,
    signing_key: SigningKey,
    peer_id: [u8; 32],
    state: Arc<RwLock<ConnectionState>>,
    sender: Arc<Mutex<Option<mpsc::Sender<Vec<u8>>>>>,
    handler: Arc<RwLock<Option<MessageHandler>>>,
    reconnect_attempts: Arc<Mutex<u32>>,
}

impl RelayClient {
    /// Create a new relay client
    pub fn new(url: &str, signing_key: SigningKey) -> Self {
        let peer_id = peer_id_bytes(&signing_key.verifying_key());
        Self {
            url: url.to_string(),
            signing_key,
            peer_id,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            sender: Arc::new(Mutex::new(None)),
            handler: Arc::new(RwLock::new(None)),
            reconnect_attempts: Arc::new(Mutex::new(0)),
        }
    }

    /// Get current connection state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Get peer ID
    pub fn peer_id(&self) -> [u8; 32] {
        self.peer_id
    }

    /// Set message handler
    pub async fn set_handler(&self, handler: MessageHandler) {
        *self.handler.write().await = Some(handler);
    }

    /// Connect to relay
    pub async fn connect(&self) -> Result<(), RelayError> {
        *self.state.write().await = ConnectionState::Connecting;

        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| RelayError::Connection(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
        *self.sender.lock().await = Some(tx);

        *self.state.write().await = ConnectionState::Authenticating;

        // Wait for auth challenge
        let challenge = match read.next().await {
            Some(Ok(WsMessage::Binary(data))) => {
                match RelayMessage::decode(&data)? {
                    RelayMessage::AuthChallenge { nonce } => nonce,
                    _ => return Err(RelayError::AuthFailed("Expected challenge".into())),
                }
            }
            _ => return Err(RelayError::AuthFailed("No challenge received".into())),
        };

        // Sign challenge and respond
        let signature = sign(&challenge, &self.signing_key);
        let response = RelayMessage::AuthResponse {
            peer_id: self.peer_id,
            signature: signature.to_bytes(),
        };
        write
            .send(WsMessage::Binary(response.encode()))
            .await
            .map_err(|e| RelayError::WebSocket(e.to_string()))?;

        // Wait for auth result
        match read.next().await {
            Some(Ok(WsMessage::Binary(data))) => {
                match RelayMessage::decode(&data)? {
                    RelayMessage::AuthSuccess => {}
                    RelayMessage::AuthFailure { reason } => {
                        return Err(RelayError::AuthFailed(reason));
                    }
                    _ => return Err(RelayError::AuthFailed("Unexpected response".into())),
                }
            }
            _ => return Err(RelayError::AuthFailed("No auth response".into())),
        }

        *self.state.write().await = ConnectionState::Connected;
        *self.reconnect_attempts.lock().await = 0;

        // Spawn writer task
        let state = self.state.clone();
        tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if write.send(WsMessage::Binary(data)).await.is_err() {
                    break;
                }
            }
            *state.write().await = ConnectionState::Disconnected;
        });

        // Spawn reader task
        let handler = self.handler.clone();
        let sender = self.sender.clone();
        let state = self.state.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(WsMessage::Binary(data)) => {
                        if let Ok(relay_msg) = RelayMessage::decode(&data) {
                            match relay_msg {
                                RelayMessage::Receive { sender: from, message_id, timestamp, payload } => {
                                    // Send ACK
                                    let ack = RelayMessage::Ack { message_id };
                                    if let Some(tx) = sender.lock().await.as_ref() {
                                        let _ = tx.send(ack.encode()).await;
                                    }
                                    // Call handler
                                    if let Some(h) = handler.read().await.as_ref() {
                                        h(IncomingMessage {
                                            sender: from,
                                            message_id,
                                            timestamp,
                                            payload,
                                        });
                                    }
                                }
                                RelayMessage::Ping => {
                                    let pong = RelayMessage::Pong;
                                    if let Some(tx) = sender.lock().await.as_ref() {
                                        let _ = tx.send(pong.encode()).await;
                                    }
                                }
                                _ => {} // Handle SendAck, PendingCount, etc. as needed
                            }
                        }
                    }
                    Ok(WsMessage::Close(_)) | Err(_) => break,
                    _ => {}
                }
            }
            *state.write().await = ConnectionState::Disconnected;
        });

        // Request pending messages
        self.fetch_pending().await?;

        Ok(())
    }

    /// Send a message to a peer
    pub async fn send(&self, recipient: [u8; 32], payload: Vec<u8>) -> Result<[u8; 16], RelayError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(RelayError::NotConnected);
        }

        let message_id: [u8; 16] = rand::random();
        let msg = RelayMessage::Send {
            recipient,
            message_id,
            payload,
        };

        if let Some(tx) = self.sender.lock().await.as_ref() {
            tx.send(msg.encode())
                .await
                .map_err(|_| RelayError::SendFailed)?;
            Ok(message_id)
        } else {
            Err(RelayError::NotConnected)
        }
    }

    /// Fetch pending messages
    pub async fn fetch_pending(&self) -> Result<(), RelayError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(RelayError::NotConnected);
        }

        let msg = RelayMessage::FetchPending;
        if let Some(tx) = self.sender.lock().await.as_ref() {
            tx.send(msg.encode())
                .await
                .map_err(|_| RelayError::SendFailed)?;
        }
        Ok(())
    }

    /// Disconnect from relay
    pub async fn disconnect(&self) {
        *self.sender.lock().await = None;
        *self.state.write().await = ConnectionState::Disconnected;
    }

    /// Reconnect with exponential backoff
    pub async fn reconnect(&self) -> Result<(), RelayError> {
        let mut attempts = self.reconnect_attempts.lock().await;
        *attempts += 1;
        let delay = Duration::from_secs(2u64.pow((*attempts).min(6)));
        drop(attempts);

        *self.state.write().await = ConnectionState::Reconnecting;
        sleep(delay).await;
        self.connect().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::keys::{generate_keypair, peer_id_bytes};

    #[test]
    fn test_client_creation() {
        let (signing, _) = generate_keypair();
        let client = RelayClient::new("ws://localhost:8080", signing);
        assert_eq!(client.state.try_read().unwrap().clone(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_peer_id_derivation() {
        let (signing, verifying) = generate_keypair();
        let client = RelayClient::new("ws://localhost:8080", signing);
        let expected = peer_id_bytes(&verifying);
        assert_eq!(client.peer_id(), expected);
    }

    #[tokio::test]
    async fn test_send_without_connection() {
        let (signing, _) = generate_keypair();
        let client = RelayClient::new("ws://localhost:8080", signing);
        let result = client.send([0u8; 32], vec![1, 2, 3]).await;
        assert!(matches!(result, Err(RelayError::NotConnected)));
    }

    #[tokio::test]
    async fn test_state_transitions() {
        let (signing, _) = generate_keypair();
        let client = RelayClient::new("ws://localhost:8080", signing);
        assert_eq!(client.state().await, ConnectionState::Disconnected);
        
        // Can't test full connect without a server, but we can test disconnect
        client.disconnect().await;
        assert_eq!(client.state().await, ConnectionState::Disconnected);
    }
}
