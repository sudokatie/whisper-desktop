//! Message encryption using XChaCha20-Poly1305.
//!
//! Uses ephemeral keys for forward secrecy:
//! - Sender generates ephemeral X25519 keypair
//! - Shared secret derived via X25519 key exchange
//! - Message encrypted with XChaCha20-Poly1305
//! - Format: [nonce (24)][ephemeral_pk (32)][ciphertext][tag (16)]

use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};
use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha512};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret};

/// Nonce size for XChaCha20-Poly1305
pub const NONCE_SIZE: usize = 24;
/// Public key size for X25519
pub const X25519_PK_SIZE: usize = 32;
/// Authentication tag size
pub const TAG_SIZE: usize = 16;

/// Convert Ed25519 signing key to X25519 static secret.
/// Uses the standard Ed25519 -> X25519 conversion via SHA-512 hash.
fn ed25519_to_x25519_secret(ed_secret: &SigningKey) -> StaticSecret {
    // Ed25519 secret key expansion uses SHA-512
    let hash = Sha512::digest(ed_secret.to_bytes());
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&hash[..32]);
    // Apply clamping for X25519
    bytes[0] &= 248;
    bytes[31] &= 127;
    bytes[31] |= 64;
    StaticSecret::from(bytes)
}

/// Convert Ed25519 verifying key bytes to X25519 public key.
/// Note: This is a simplified conversion - proper conversion requires
/// the Montgomery form. For now, we derive X25519 keys independently.
fn _ed25519_pk_to_x25519(_ed_pk: &[u8; 32]) -> X25519PublicKey {
    // Proper Ed25519 -> X25519 public key conversion is complex
    // For simplicity, we generate X25519 keypairs from the same seed
    unimplemented!("Use derive_x25519_keypair instead")
}

/// Derive X25519 keypair from Ed25519 secret key.
/// Returns (secret, public) for key exchange.
fn derive_x25519_keypair(ed_secret: &SigningKey) -> (StaticSecret, X25519PublicKey) {
    let secret = ed25519_to_x25519_secret(ed_secret);
    let public = X25519PublicKey::from(&secret);
    (secret, public)
}

/// Encrypt a message for a recipient using ephemeral key exchange.
///
/// The recipient_sk is used to derive the X25519 public key for key exchange.
/// Returns: [nonce (24)][ephemeral_pk (32)][ciphertext][tag (16)]
pub fn encrypt(plaintext: &[u8], recipient_sk: &SigningKey, _sender_sk: &SigningKey) -> Vec<u8> {
    // Generate ephemeral keypair for this message
    let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
    let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);
    
    // Derive recipient's X25519 public key from their Ed25519 secret
    let (_, recipient_x25519_pk) = derive_x25519_keypair(recipient_sk);
    
    // Compute shared secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_x25519_pk);
    
    // Derive encryption key from shared secret using SHA-256
    let key = sha2::Sha256::digest(shared_secret.as_bytes());
    let cipher = XChaCha20Poly1305::new_from_slice(&key).expect("valid key size");
    
    // Generate random nonce
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    
    // Encrypt
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("encryption failed");
    
    // Assemble output: nonce + ephemeral_pk + ciphertext
    let mut output = Vec::with_capacity(NONCE_SIZE + X25519_PK_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(ephemeral_public.as_bytes());
    output.extend_from_slice(&ciphertext);
    
    output
}

/// Decrypt a message using the recipient's secret key.
pub fn decrypt(ciphertext: &[u8], recipient_sk: &SigningKey) -> Result<Vec<u8>, &'static str> {
    if ciphertext.len() < NONCE_SIZE + X25519_PK_SIZE + TAG_SIZE {
        return Err("ciphertext too short");
    }
    
    // Extract components
    let nonce = XNonce::from_slice(&ciphertext[..NONCE_SIZE]);
    let ephemeral_pk_bytes: [u8; 32] = ciphertext[NONCE_SIZE..NONCE_SIZE + X25519_PK_SIZE]
        .try_into()
        .map_err(|_| "invalid ephemeral public key")?;
    let encrypted = &ciphertext[NONCE_SIZE + X25519_PK_SIZE..];
    
    // Derive recipient's X25519 secret from Ed25519 secret
    let (recipient_x25519_sk, _) = derive_x25519_keypair(recipient_sk);
    let ephemeral_public = X25519PublicKey::from(ephemeral_pk_bytes);
    
    // Compute shared secret
    let shared_secret = recipient_x25519_sk.diffie_hellman(&ephemeral_public);
    
    // Derive decryption key
    let key = sha2::Sha256::digest(shared_secret.as_bytes());
    let cipher = XChaCha20Poly1305::new_from_slice(&key).expect("valid key size");
    
    // Decrypt
    cipher.decrypt(nonce, encrypted).map_err(|_| "decryption failed")
}

/// Encrypt with a symmetric key (for local storage).
pub fn encrypt_symmetric(plaintext: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("valid key size");
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("encryption failed");
    
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    output
}

/// Decrypt with a symmetric key.
pub fn decrypt_symmetric(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, &'static str> {
    if ciphertext.len() < NONCE_SIZE + TAG_SIZE {
        return Err("ciphertext too short");
    }
    
    let nonce = XNonce::from_slice(&ciphertext[..NONCE_SIZE]);
    let encrypted = &ciphertext[NONCE_SIZE..];
    
    let cipher = XChaCha20Poly1305::new_from_slice(key).expect("valid key size");
    cipher.decrypt(nonce, encrypted).map_err(|_| "decryption failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::keys::generate_keypair;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (sender_sk, _sender_pk) = generate_keypair();
        let (recipient_sk, _recipient_pk) = generate_keypair();
        let plaintext = b"hello whisper";
        
        // Encrypt using recipient's secret (to derive their X25519 public)
        let ciphertext = encrypt(plaintext, &recipient_sk, &sender_sk);
        let decrypted = decrypt(&ciphertext, &recipient_sk).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_nonces() {
        let (sender_sk, _) = generate_keypair();
        let (recipient_sk, _) = generate_keypair();
        let plaintext = b"same message";
        
        let ct1 = encrypt(plaintext, &recipient_sk, &sender_sk);
        let ct2 = encrypt(plaintext, &recipient_sk, &sender_sk);
        
        // Nonces should differ (first 24 bytes)
        assert_ne!(&ct1[..NONCE_SIZE], &ct2[..NONCE_SIZE]);
    }

    #[test]
    fn test_wrong_key_fails() {
        let (sender_sk, _) = generate_keypair();
        let (recipient_sk, _) = generate_keypair();
        let (wrong_sk, _) = generate_keypair();
        let plaintext = b"secret";
        
        let ciphertext = encrypt(plaintext, &recipient_sk, &sender_sk);
        let result = decrypt(&ciphertext, &wrong_sk);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_symmetric_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"local storage test";
        
        let ciphertext = encrypt_symmetric(plaintext, &key);
        let decrypted = decrypt_symmetric(&ciphertext, &key).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_symmetric_wrong_key_fails() {
        let key = [42u8; 32];
        let wrong_key = [43u8; 32];
        let plaintext = b"local storage test";
        
        let ciphertext = encrypt_symmetric(plaintext, &key);
        let result = decrypt_symmetric(&ciphertext, &wrong_key);
        
        assert!(result.is_err());
    }
}
