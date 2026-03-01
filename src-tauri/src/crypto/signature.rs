//! Ed25519 signatures for message authentication.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// Sign a message with the secret key.
pub fn sign(message: &[u8], secret: &SigningKey) -> Signature {
    secret.sign(message)
}

/// Verify a signature against a message and public key.
pub fn verify(message: &[u8], signature: &Signature, public: &VerifyingKey) -> bool {
    public.verify(message, signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::keys::generate_keypair;

    #[test]
    fn test_sign_verify_roundtrip() {
        let (signing, verifying) = generate_keypair();
        let message = b"test message";
        
        let signature = sign(message, &signing);
        assert!(verify(message, &signature, &verifying));
    }

    #[test]
    fn test_wrong_key_fails() {
        let (signing, _) = generate_keypair();
        let (_, other_verifying) = generate_keypair();
        let message = b"test message";
        
        let signature = sign(message, &signing);
        assert!(!verify(message, &signature, &other_verifying));
    }

    #[test]
    fn test_tampered_message_fails() {
        let (signing, verifying) = generate_keypair();
        let message = b"test message";
        let tampered = b"test messag3";
        
        let signature = sign(message, &signing);
        assert!(!verify(tampered, &signature, &verifying));
    }
}
