//! Cryptographic primitives for Whisper messaging.
//!
//! - Ed25519 for signing and identity
//! - X25519 for key exchange
//! - XChaCha20-Poly1305 for encryption
//! - Argon2id for passphrase key derivation

pub mod keys;
pub mod encryption;
pub mod signature;

pub use keys::*;
