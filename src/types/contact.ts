/**
 * Contact types for Whisper Desktop.
 */

/** Trust level for a contact. */
export type TrustLevel = 'unknown' | 'unverified' | 'verified';

/** A contact entry. */
export interface Contact {
  /** Base58-encoded peer ID */
  peerId: string;
  /** User-assigned alias */
  alias: string;
  /** Hex-encoded Ed25519 public key */
  publicKey: string;
  /** Trust level based on verification */
  trustLevel: TrustLevel;
  /** When the contact was added */
  createdAt: number;
  /** Last updated timestamp */
  updatedAt: number;
}

/** Contact share data (for QR codes). */
export interface ContactShare {
  /** Peer ID */
  peerId: string;
  /** Public key */
  publicKey: string;
  /** Optional alias */
  alias?: string;
}

/** New contact input. */
export interface NewContact {
  peerId: string;
  alias: string;
  publicKey: string;
}
