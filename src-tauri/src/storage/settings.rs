//! Settings storage.

use crate::storage::database::Database;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use thiserror::Error;

/// Settings storage errors.
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// App settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Theme: "light", "dark", or "system"
    pub theme: String,
    /// Show notifications
    pub notifications: bool,
    /// Notification sound
    pub sound: bool,
    /// Start on login
    pub autostart: bool,
    /// Minimize to tray on close
    pub minimize_to_tray: bool,
    /// Relay URL
    pub relay_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            notifications: true,
            sound: true,
            autostart: false,
            minimize_to_tray: true,
            relay_url: "wss://relay.whisper.dev".to_string(),
        }
    }
}

/// Get current settings.
pub async fn get_settings(db: &Database) -> Result<Settings, SettingsError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = 'app'")
        .fetch_optional(db.pool())
        .await?;
    
    match row {
        Some(r) => {
            let value: String = r.get("value");
            Ok(serde_json::from_str(&value)?)
        }
        None => Ok(Settings::default()),
    }
}

/// Update settings.
pub async fn update_settings(db: &Database, settings: &Settings) -> Result<(), SettingsError> {
    let value = serde_json::to_string(settings)?;
    
    sqlx::query(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('app', ?)",
    )
    .bind(&value)
    .execute(db.pool())
    .await?;
    
    Ok(())
}

/// Get a single setting by key.
pub async fn get_setting(db: &Database, key: &str) -> Result<Option<String>, SettingsError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(db.pool())
        .await?;
    
    Ok(row.map(|r| r.get("value")))
}

/// Set a single setting.
pub async fn set_setting(db: &Database, key: &str, value: &str) -> Result<(), SettingsError> {
    sqlx::query(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
    )
    .bind(key)
    .bind(value)
    .execute(db.pool())
    .await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_settings() {
        let db = Database::open_memory().await.unwrap();
        let settings = get_settings(&db).await.unwrap();
        
        assert_eq!(settings.theme, "system");
        assert!(settings.notifications);
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_update_settings() {
        let db = Database::open_memory().await.unwrap();
        
        let mut settings = Settings::default();
        settings.theme = "dark".to_string();
        settings.notifications = false;
        
        update_settings(&db, &settings).await.unwrap();
        
        let loaded = get_settings(&db).await.unwrap();
        assert_eq!(loaded.theme, "dark");
        assert!(!loaded.notifications);
        
        db.close().await;
    }

    #[tokio::test]
    async fn test_single_setting() {
        let db = Database::open_memory().await.unwrap();
        
        set_setting(&db, "custom_key", "custom_value").await.unwrap();
        let value = get_setting(&db, "custom_key").await.unwrap();
        
        assert_eq!(value, Some("custom_value".to_string()));
        
        db.close().await;
    }
}
