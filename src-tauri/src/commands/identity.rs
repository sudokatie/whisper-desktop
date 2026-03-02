//! Identity commands.

use crate::commands::AppState;
use serde::Serialize;
use tauri::State;

/// Identity info returned to frontend
#[derive(Debug, Clone, Serialize)]
pub struct IdentityInfo {
    pub peer_id: String,
    pub public_key: String,
    pub is_locked: bool,
    pub created_at: i64,
}

/// Error wrapper for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl<E: std::fmt::Display> From<E> for CommandError {
    fn from(e: E) -> Self {
        Self { message: e.to_string() }
    }
}

/// Create a new identity with the given passphrase.
#[tauri::command]
pub async fn create_identity(
    state: State<'_, AppState>,
    passphrase: String,
) -> Result<IdentityInfo, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    let stored = crate::storage::identity::create_identity(db, &passphrase)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?;
    
    // Auto-unlock after creation
    let signing_key = crate::storage::identity::unlock_identity(db, &passphrase)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?;
    
    *state.signing_key.lock().await = Some(signing_key);
    
    Ok(IdentityInfo {
        peer_id: stored.peer_id,
        public_key: hex::encode(&stored.public_key),
        is_locked: false,
        created_at: stored.created_at,
    })
}

/// Get current identity info.
#[tauri::command]
pub async fn get_identity(
    state: State<'_, AppState>,
) -> Result<Option<IdentityInfo>, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    let stored = crate::storage::identity::get_identity(db)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?;
    
    let is_locked = state.signing_key.lock().await.is_none();
    
    Ok(stored.map(|s| IdentityInfo {
        peer_id: s.peer_id,
        public_key: hex::encode(&s.public_key),
        is_locked,
        created_at: s.created_at,
    }))
}

/// Unlock the identity with passphrase.
#[tauri::command]
pub async fn unlock(
    state: State<'_, AppState>,
    passphrase: String,
) -> Result<IdentityInfo, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    let signing_key = crate::storage::identity::unlock_identity(db, &passphrase)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?;
    
    let stored = crate::storage::identity::get_identity(db)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?
        .ok_or_else(|| CommandError { message: "Identity not found".into() })?;
    
    *state.signing_key.lock().await = Some(signing_key);
    
    // TODO: Connect to relay
    
    Ok(IdentityInfo {
        peer_id: stored.peer_id,
        public_key: hex::encode(&stored.public_key),
        is_locked: false,
        created_at: stored.created_at,
    })
}

/// Lock the identity (clear signing key).
#[tauri::command]
pub async fn lock(state: State<'_, AppState>) -> Result<(), CommandError> {
    *state.signing_key.lock().await = None;
    // TODO: Disconnect from relay
    Ok(())
}

/// Change the passphrase.
#[tauri::command]
pub async fn change_passphrase(
    state: State<'_, AppState>,
    old_passphrase: String,
    new_passphrase: String,
) -> Result<(), CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::identity::change_passphrase(db, &old_passphrase, &new_passphrase)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?;
    
    Ok(())
}

/// Generate QR code data for linking (peer_id + public_key).
#[tauri::command]
pub async fn get_link_qr(
    state: State<'_, AppState>,
) -> Result<String, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    let stored = crate::storage::identity::get_identity(db)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?
        .ok_or_else(|| CommandError { message: "Identity not found".into() })?;
    
    // Format: whisper://<peer_id>/<public_key_hex>
    Ok(format!("whisper://{}/{}", stored.peer_id, hex::encode(&stored.public_key)))
}
