//! Contact storage.
//!
//! CRUD operations for managing contacts.

use crate::storage::database::Database;
use sqlx::Row;
use thiserror::Error;

/// Contact storage errors.
#[derive(Debug, Error)]
pub enum ContactError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Contact not found")]
    NotFound,
    #[error("Contact already exists")]
    AlreadyExists,
}

/// Trust level for a contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Unknown,
    Unverified,
    Verified,
}

impl TrustLevel {
    fn as_str(&self) -> &'static str {
        match self {
            TrustLevel::Unknown => "unknown",
            TrustLevel::Unverified => "unverified",
            TrustLevel::Verified => "verified",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "verified" => TrustLevel::Verified,
            "unverified" => TrustLevel::Unverified,
            _ => TrustLevel::Unknown,
        }
    }
}

/// A contact entry.
#[derive(Debug, Clone)]
pub struct Contact {
    pub peer_id: String,
    pub alias: String,
    pub public_key: Vec<u8>,
    pub trust_level: TrustLevel,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Add a new contact.
pub async fn add_contact(db: &Database, contact: &Contact) -> Result<(), ContactError> {
    // Check if contact already exists
    let existing = sqlx::query("SELECT peer_id FROM contacts WHERE peer_id = ?")
        .bind(&contact.peer_id)
        .fetch_optional(db.pool())
        .await?;

    if existing.is_some() {
        return Err(ContactError::AlreadyExists);
    }

    sqlx::query(
        "INSERT INTO contacts (peer_id, alias, public_key, trust_level, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&contact.peer_id)
    .bind(&contact.alias)
    .bind(&contact.public_key)
    .bind(contact.trust_level.as_str())
    .bind(contact.created_at)
    .bind(contact.updated_at)
    .execute(db.pool())
    .await?;

    Ok(())
}

/// Get all contacts.
pub async fn get_contacts(db: &Database) -> Result<Vec<Contact>, ContactError> {
    let rows = sqlx::query(
        "SELECT peer_id, alias, public_key, trust_level, created_at, updated_at FROM contacts ORDER BY alias",
    )
    .fetch_all(db.pool())
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Contact {
            peer_id: row.get("peer_id"),
            alias: row.get("alias"),
            public_key: row.get("public_key"),
            trust_level: TrustLevel::from_str(row.get("trust_level")),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .collect())
}

/// Get a single contact by peer ID.
pub async fn get_contact(db: &Database, peer_id: &str) -> Result<Option<Contact>, ContactError> {
    let row = sqlx::query(
        "SELECT peer_id, alias, public_key, trust_level, created_at, updated_at FROM contacts WHERE peer_id = ?",
    )
    .bind(peer_id)
    .fetch_optional(db.pool())
    .await?;

    Ok(row.map(|r| Contact {
        peer_id: r.get("peer_id"),
        alias: r.get("alias"),
        public_key: r.get("public_key"),
        trust_level: TrustLevel::from_str(r.get("trust_level")),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}

/// Update a contact.
pub async fn update_contact(db: &Database, contact: &Contact) -> Result<(), ContactError> {
    let result = sqlx::query(
        "UPDATE contacts SET alias = ?, public_key = ?, trust_level = ?, updated_at = ? WHERE peer_id = ?",
    )
    .bind(&contact.alias)
    .bind(&contact.public_key)
    .bind(contact.trust_level.as_str())
    .bind(contact.updated_at)
    .bind(&contact.peer_id)
    .execute(db.pool())
    .await?;

    if result.rows_affected() == 0 {
        return Err(ContactError::NotFound);
    }

    Ok(())
}

/// Delete a contact.
pub async fn delete_contact(db: &Database, peer_id: &str) -> Result<(), ContactError> {
    let result = sqlx::query("DELETE FROM contacts WHERE peer_id = ?")
        .bind(peer_id)
        .execute(db.pool())
        .await?;

    if result.rows_affected() == 0 {
        return Err(ContactError::NotFound);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_contact() -> Contact {
        Contact {
            peer_id: "test_peer_id".to_string(),
            alias: "Alice".to_string(),
            public_key: vec![1, 2, 3, 4],
            trust_level: TrustLevel::Unknown,
            created_at: 1000,
            updated_at: 1000,
        }
    }

    #[tokio::test]
    async fn test_add_contact() {
        let db = Database::open_memory().await.unwrap();
        let contact = test_contact();
        
        add_contact(&db, &contact).await.unwrap();
        
        let fetched = get_contact(&db, &contact.peer_id).await.unwrap().unwrap();
        assert_eq!(fetched.alias, "Alice");
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_add_duplicate_contact() {
        let db = Database::open_memory().await.unwrap();
        let contact = test_contact();
        
        add_contact(&db, &contact).await.unwrap();
        let result = add_contact(&db, &contact).await;
        
        assert!(matches!(result, Err(ContactError::AlreadyExists)));
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_get_contacts() {
        let db = Database::open_memory().await.unwrap();
        
        let mut c1 = test_contact();
        c1.peer_id = "peer1".to_string();
        c1.alias = "Alice".to_string();
        
        let mut c2 = test_contact();
        c2.peer_id = "peer2".to_string();
        c2.alias = "Bob".to_string();
        
        add_contact(&db, &c1).await.unwrap();
        add_contact(&db, &c2).await.unwrap();
        
        let contacts = get_contacts(&db).await.unwrap();
        assert_eq!(contacts.len(), 2);
        // Should be sorted by alias
        assert_eq!(contacts[0].alias, "Alice");
        assert_eq!(contacts[1].alias, "Bob");
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_update_contact() {
        let db = Database::open_memory().await.unwrap();
        let mut contact = test_contact();
        
        add_contact(&db, &contact).await.unwrap();
        
        contact.alias = "Alice Updated".to_string();
        contact.trust_level = TrustLevel::Verified;
        update_contact(&db, &contact).await.unwrap();
        
        let fetched = get_contact(&db, &contact.peer_id).await.unwrap().unwrap();
        assert_eq!(fetched.alias, "Alice Updated");
        assert_eq!(fetched.trust_level, TrustLevel::Verified);
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_delete_contact() {
        let db = Database::open_memory().await.unwrap();
        let contact = test_contact();
        
        add_contact(&db, &contact).await.unwrap();
        delete_contact(&db, &contact.peer_id).await.unwrap();
        
        let fetched = get_contact(&db, &contact.peer_id).await.unwrap();
        assert!(fetched.is_none());
        
        db.close().await;
    }
}
