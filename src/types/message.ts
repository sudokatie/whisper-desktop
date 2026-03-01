/**
 * Message types for Whisper Desktop.
 */

/** Message delivery status. */
export type MessageStatus = 'pending' | 'sent' | 'delivered' | 'read' | 'failed';

/** Message direction. */
export type MessageDirection = 'incoming' | 'outgoing';

/** A single message. */
export interface Message {
  /** Unique message ID (UUID) */
  id: string;
  /** Peer ID of the other party */
  peerId: string;
  /** Decrypted message content */
  content: string;
  /** Unix timestamp */
  timestamp: number;
  /** Delivery status */
  status: MessageStatus;
  /** Direction */
  direction: MessageDirection;
}

/** Conversation summary (for list view). */
export interface Conversation {
  /** Peer ID of the other party */
  peerId: string;
  /** Contact alias if known */
  alias?: string;
  /** Last message preview */
  lastMessage?: string;
  /** Last message timestamp */
  lastMessageAt?: number;
  /** Number of unread messages */
  unreadCount: number;
}

/** Encrypted message as stored/transmitted. */
export interface EncryptedMessage {
  id: string;
  /** Sender's peer ID */
  from: string;
  /** Recipient's peer ID */
  to: string;
  /** Base64-encoded ciphertext */
  ciphertext: string;
  /** Unix timestamp */
  timestamp: number;
}

/** Message for sending. */
export interface OutgoingMessage {
  /** Recipient's peer ID */
  peerId: string;
  /** Plaintext content */
  content: string;
}
