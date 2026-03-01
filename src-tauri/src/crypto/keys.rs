//! Key generation and management.

use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

/// Generate a new Ed25519 keypair.
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Derive peer ID from public key (Base58 encoded string).
pub fn derive_peer_id(public_key: &VerifyingKey) -> String {
    bs58::encode(public_key.as_bytes()).into_string()
}

/// Get peer ID as raw bytes (for wire protocol).
pub fn peer_id_bytes(public_key: &VerifyingKey) -> [u8; 32] {
    *public_key.as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (signing, verifying) = generate_keypair();
        assert_eq!(signing.verifying_key(), verifying);
    }

    #[test]
    fn test_peer_id_deterministic() {
        let (_, verifying) = generate_keypair();
        let id1 = derive_peer_id(&verifying);
        let id2 = derive_peer_id(&verifying);
        assert_eq!(id1, id2);
    }
}
