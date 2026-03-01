/**
 * Identity types for Whisper Desktop.
 */

/** User identity with key pair. */
export interface Identity {
  /** Base58-encoded peer ID derived from public key */
  peerId: string;
  /** Hex-encoded Ed25519 public key */
  publicKey: string;
  /** Unix timestamp of creation */
  createdAt: number;
}

/** Encrypted identity for storage. */
export interface EncryptedIdentity {
  peerId: string;
  publicKey: string;
  /** XChaCha20-Poly1305 encrypted secret key */
  encryptedSecret: string;
  /** Argon2id salt (hex) */
  salt: string;
  createdAt: number;
}

/** Identity state for the frontend. */
export type IdentityState = 
  | { status: 'locked' }
  | { status: 'unlocked'; identity: Identity }
  | { status: 'none' };

/** Device linking request. */
export interface LinkRequest {
  /** Challenge to sign */
  challenge: string;
  /** Requesting device's peer ID */
  requesterPeerId: string;
  /** Unix timestamp of request */
  timestamp: number;
}

/** Device link confirmation. */
export interface LinkConfirmation {
  /** Signed challenge */
  signature: string;
  /** Confirming device's peer ID */
  confirmerPeerId: string;
  /** Confirming device's public key */
  confirmerPublicKey: string;
}
