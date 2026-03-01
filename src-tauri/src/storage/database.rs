//! Database connection and migrations.
//!
//! Uses SQLite with passphrase-based encryption via sqlx.
//! Note: SQLCipher requires bundled feature which has native deps.
//! For now, we use standard SQLite with application-level encryption.

use sqlx::{sqlite::SqlitePool, Pool, Sqlite};
use std::path::Path;
use thiserror::Error;

/// Database errors.
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Database not initialized")]
    NotInitialized,
}

/// Database connection wrapper.
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    /// Open database at the given path.
    /// Creates the database file if it doesn't exist.
    pub async fn open(path: &Path) -> Result<Self, DatabaseError> {
        let url = format!("sqlite:{}?mode=rwc", path.display());
        let pool = SqlitePool::connect(&url).await?;
        
        let db = Self { pool };
        db.migrate().await?;
        
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub async fn open_memory() -> Result<Self, DatabaseError> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        
        let db = Self { pool };
        db.migrate().await?;
        
        Ok(db)
    }

    /// Run database migrations.
    async fn migrate(&self) -> Result<(), DatabaseError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS identity (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                peer_id TEXT NOT NULL,
                public_key BLOB NOT NULL,
                encrypted_secret BLOB NOT NULL,
                salt BLOB NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS contacts (
                peer_id TEXT PRIMARY KEY,
                alias TEXT NOT NULL,
                public_key BLOB NOT NULL,
                trust_level TEXT NOT NULL DEFAULT 'unknown',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                peer_id TEXT NOT NULL,
                content BLOB NOT NULL,
                timestamp INTEGER NOT NULL,
                status TEXT NOT NULL,
                direction TEXT NOT NULL,
                FOREIGN KEY (peer_id) REFERENCES contacts(peer_id)
            );

            CREATE INDEX IF NOT EXISTS idx_messages_peer_id ON messages(peer_id);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        Ok(())
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Close the database connection.
    pub async fn close(self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_open_memory() {
        let db = Database::open_memory().await.unwrap();
        assert!(!db.pool.is_closed());
        db.close().await;
    }

    #[tokio::test]
    async fn test_migrations_run() {
        let db = Database::open_memory().await.unwrap();
        
        // Check that tables exist
        let result = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name='identity'")
            .fetch_optional(db.pool())
            .await
            .unwrap();
        
        assert!(result.is_some());
        db.close().await;
    }

    #[tokio::test]
    async fn test_migrations_idempotent() {
        let db = Database::open_memory().await.unwrap();
        
        // Running migrate again should not error
        db.migrate().await.unwrap();
        db.migrate().await.unwrap();
        
        db.close().await;
    }
}
