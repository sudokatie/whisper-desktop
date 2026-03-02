//! Contact commands.

use crate::commands::AppState;
use crate::storage::contacts::{Contact, TrustLevel};
use serde::Serialize;
use tauri::State;

/// Error wrapper for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

/// Get all contacts.
#[tauri::command]
pub async fn get_contacts(
    state: State<'_, AppState>,
) -> Result<Vec<Contact>, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::contacts::get_contacts(db)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Add a new contact.
#[tauri::command]
pub async fn add_contact(
    state: State<'_, AppState>,
    peer_id: String,
    alias: Option<String>,
    public_key: Vec<u8>,
) -> Result<Contact, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    let now = chrono::Utc::now().timestamp();
    let contact = Contact {
        peer_id: peer_id.clone(),
        alias: alias.unwrap_or_else(|| peer_id[..8.min(peer_id.len())].to_string()),
        public_key,
        trust_level: TrustLevel::Unknown,
        created_at: now,
        updated_at: now,
    };
    
    crate::storage::contacts::add_contact(db, &contact)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?;
    
    Ok(contact)
}

/// Add a contact from QR code data.
#[tauri::command]
pub async fn add_contact_from_qr(
    state: State<'_, AppState>,
    qr_data: String,
    alias: Option<String>,
) -> Result<Contact, CommandError> {
    // Parse: whisper://<peer_id>/<public_key_hex>
    let parts: Vec<&str> = qr_data
        .strip_prefix("whisper://")
        .ok_or_else(|| CommandError { message: "Invalid QR format".into() })?
        .split('/')
        .collect();
    
    if parts.len() != 2 {
        return Err(CommandError { message: "Invalid QR format".into() });
    }
    
    let peer_id = parts[0].to_string();
    let public_key = hex::decode(parts[1])
        .map_err(|e| CommandError { message: format!("Invalid public key: {}", e) })?;
    
    add_contact(state, peer_id, alias, public_key).await
}

/// Update contact alias.
#[tauri::command]
pub async fn update_contact_alias(
    state: State<'_, AppState>,
    peer_id: String,
    alias: String,
) -> Result<(), CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    let mut contact = crate::storage::contacts::get_contact(db, &peer_id)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?
        .ok_or_else(|| CommandError { message: "Contact not found".into() })?;
    
    contact.alias = alias;
    contact.updated_at = chrono::Utc::now().timestamp();
    
    crate::storage::contacts::update_contact(db, &contact)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Update contact trust level.
#[tauri::command]
pub async fn update_contact_trust(
    state: State<'_, AppState>,
    peer_id: String,
    trust_level: String,
) -> Result<(), CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    let mut contact = crate::storage::contacts::get_contact(db, &peer_id)
        .await
        .map_err(|e| CommandError { message: e.to_string() })?
        .ok_or_else(|| CommandError { message: "Contact not found".into() })?;
    
    contact.trust_level = match trust_level.as_str() {
        "verified" => TrustLevel::Verified,
        "unverified" => TrustLevel::Unverified,
        _ => TrustLevel::Unknown,
    };
    contact.updated_at = chrono::Utc::now().timestamp();
    
    crate::storage::contacts::update_contact(db, &contact)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Delete a contact.
#[tauri::command]
pub async fn delete_contact(
    state: State<'_, AppState>,
    peer_id: String,
) -> Result<(), CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::contacts::delete_contact(db, &peer_id)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}
