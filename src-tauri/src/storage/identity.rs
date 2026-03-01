//! Identity storage and management.
//!
//! Handles encrypted storage of Ed25519 keypairs using Argon2id
//! for passphrase-based key derivation.

use crate::crypto::{
    encryption::{decrypt_symmetric, encrypt_symmetric},
    keys::{derive_peer_id, generate_keypair},
};
use crate::storage::database::Database;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use ed25519_dalek::SigningKey;
use sqlx::Row;
use thiserror::Error;

/// Identity storage errors.
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Identity already exists")]
    AlreadyExists,
    #[error("Identity not found")]
    NotFound,
    #[error("Invalid passphrase")]
    InvalidPassphrase,
    #[error("Crypto error: {0}")]
    Crypto(String),
}

/// Stored identity data.
#[derive(Debug, Clone)]
pub struct StoredIdentity {
    pub peer_id: String,
    pub public_key: Vec<u8>,
    pub created_at: i64,
}

/// Create a new identity with the given passphrase.
pub async fn create_identity(
    db: &Database,
    passphrase: &str,
) -> Result<StoredIdentity, IdentityError> {
    // Check if identity already exists
    let existing = sqlx::query("SELECT id FROM identity WHERE id = 1")
        .fetch_optional(db.pool())
        .await?;
    
    if existing.is_some() {
        return Err(IdentityError::AlreadyExists);
    }

    // Generate keypair
    let (signing_key, verifying_key) = generate_keypair();
    let peer_id = derive_peer_id(&verifying_key);
    let public_key = verifying_key.as_bytes().to_vec();

    // Generate salt and derive encryption key
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| IdentityError::Crypto(e.to_string()))?;
    
    // Use first 32 bytes of hash output as encryption key
    let hash_str = hash.hash.unwrap().to_string();
    let mut key = [0u8; 32];
    let hash_bytes = hash_str.as_bytes();
    key.copy_from_slice(&hash_bytes[..32.min(hash_bytes.len())]);

    // Encrypt the secret key
    let encrypted_secret = encrypt_symmetric(signing_key.as_bytes(), &key);

    let created_at = chrono::Utc::now().timestamp();

    // Store in database
    sqlx::query(
        "INSERT INTO identity (id, peer_id, public_key, encrypted_secret, salt, created_at) VALUES (1, ?, ?, ?, ?, ?)",
    )
    .bind(&peer_id)
    .bind(&public_key)
    .bind(&encrypted_secret)
    .bind(salt.as_str())
    .bind(created_at)
    .execute(db.pool())
    .await?;

    Ok(StoredIdentity {
        peer_id,
        public_key,
        created_at,
    })
}

/// Get the stored identity (without secret key).
pub async fn get_identity(db: &Database) -> Result<Option<StoredIdentity>, IdentityError> {
    let row = sqlx::query("SELECT peer_id, public_key, created_at FROM identity WHERE id = 1")
        .fetch_optional(db.pool())
        .await?;

    Ok(row.map(|r| StoredIdentity {
        peer_id: r.get("peer_id"),
        public_key: r.get("public_key"),
        created_at: r.get("created_at"),
    }))
}

/// Unlock the identity with the passphrase, returning the signing key.
pub async fn unlock_identity(
    db: &Database,
    passphrase: &str,
) -> Result<SigningKey, IdentityError> {
    let row = sqlx::query("SELECT encrypted_secret, salt FROM identity WHERE id = 1")
        .fetch_optional(db.pool())
        .await?
        .ok_or(IdentityError::NotFound)?;

    let encrypted_secret: Vec<u8> = row.get("encrypted_secret");
    let salt_str: String = row.get("salt");

    // Derive key from passphrase
    let salt = SaltString::from_b64(&salt_str)
        .map_err(|e| IdentityError::Crypto(e.to_string()))?;
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| IdentityError::Crypto(e.to_string()))?;

    let hash_str = hash.hash.unwrap().to_string();
    let mut key = [0u8; 32];
    let hash_bytes = hash_str.as_bytes();
    key.copy_from_slice(&hash_bytes[..32.min(hash_bytes.len())]);

    // Decrypt the secret key
    let secret_bytes = decrypt_symmetric(&encrypted_secret, &key)
        .map_err(|_| IdentityError::InvalidPassphrase)?;

    let secret_array: [u8; 32] = secret_bytes
        .try_into()
        .map_err(|_| IdentityError::Crypto("Invalid key length".to_string()))?;

    Ok(SigningKey::from_bytes(&secret_array))
}

/// Change the identity passphrase.
pub async fn change_passphrase(
    db: &Database,
    old_passphrase: &str,
    new_passphrase: &str,
) -> Result<(), IdentityError> {
    // First unlock with old passphrase
    let signing_key = unlock_identity(db, old_passphrase).await?;

    // Generate new salt and derive new key
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(new_passphrase.as_bytes(), &salt)
        .map_err(|e| IdentityError::Crypto(e.to_string()))?;

    let hash_str = hash.hash.unwrap().to_string();
    let mut key = [0u8; 32];
    let hash_bytes = hash_str.as_bytes();
    key.copy_from_slice(&hash_bytes[..32.min(hash_bytes.len())]);

    // Re-encrypt with new key
    let encrypted_secret = encrypt_symmetric(signing_key.as_bytes(), &key);

    // Update database
    sqlx::query("UPDATE identity SET encrypted_secret = ?, salt = ? WHERE id = 1")
        .bind(&encrypted_secret)
        .bind(salt.as_str())
        .execute(db.pool())
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_identity() {
        let db = Database::open_memory().await.unwrap();
        let identity = create_identity(&db, "test123").await.unwrap();
        
        assert!(!identity.peer_id.is_empty());
        assert_eq!(identity.public_key.len(), 32);
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_create_identity_already_exists() {
        let db = Database::open_memory().await.unwrap();
        create_identity(&db, "test123").await.unwrap();
        
        let result = create_identity(&db, "test456").await;
        assert!(matches!(result, Err(IdentityError::AlreadyExists)));
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_get_identity() {
        let db = Database::open_memory().await.unwrap();
        let created = create_identity(&db, "test123").await.unwrap();
        
        let fetched = get_identity(&db).await.unwrap().unwrap();
        assert_eq!(fetched.peer_id, created.peer_id);
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_unlock_identity() {
        let db = Database::open_memory().await.unwrap();
        create_identity(&db, "test123").await.unwrap();
        
        let signing_key = unlock_identity(&db, "test123").await.unwrap();
        assert_eq!(signing_key.as_bytes().len(), 32);
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_unlock_wrong_passphrase() {
        let db = Database::open_memory().await.unwrap();
        create_identity(&db, "test123").await.unwrap();
        
        let result = unlock_identity(&db, "wrong").await;
        assert!(matches!(result, Err(IdentityError::InvalidPassphrase)));
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_change_passphrase() {
        let db = Database::open_memory().await.unwrap();
        create_identity(&db, "old_pass").await.unwrap();
        
        change_passphrase(&db, "old_pass", "new_pass").await.unwrap();
        
        // Old passphrase should fail
        let result = unlock_identity(&db, "old_pass").await;
        assert!(matches!(result, Err(IdentityError::InvalidPassphrase)));
        
        // New passphrase should work
        let signing_key = unlock_identity(&db, "new_pass").await.unwrap();
        assert_eq!(signing_key.as_bytes().len(), 32);
        
        db.close().await;
    }
}
