//! Message commands.

use crate::commands::AppState;
use crate::storage::messages::{Conversation, Message, MessageDirection};
use serde::Serialize;
use tauri::{Emitter, State};

/// Error wrapper for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

/// Get all conversations with last message.
#[tauri::command]
pub async fn get_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<Conversation>, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::messages::get_conversations(db)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Get messages for a peer with pagination.
#[tauri::command]
pub async fn get_messages(
    state: State<'_, AppState>,
    peer_id: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<Message>, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::messages::get_messages(db, &peer_id, limit.unwrap_or(50), offset.unwrap_or(0))
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Send a message to a peer.
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    peer_id: String,
    content: String,
    app_handle: tauri::AppHandle,
) -> Result<Message, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    // Store message locally first
    let msg = crate::storage::messages::create_message(
        db,
        &peer_id,
        &content,
        MessageDirection::Outgoing,
    )
    .await
    .map_err(|e| CommandError { message: e.to_string() })?;
    
    // Try to send via relay
    let relay = state.relay.lock().await;
    if let Some(client) = relay.as_ref() {
        // TODO: Encrypt message before sending
        let payload = content.as_bytes().to_vec();
        
        // Parse peer_id to bytes
        let recipient = bs58::decode(&peer_id)
            .into_vec()
            .map_err(|e| CommandError { message: format!("Invalid peer_id: {}", e) })?;
        
        if recipient.len() != 32 {
            return Err(CommandError { message: "Invalid peer_id length".into() });
        }
        
        let mut recipient_arr = [0u8; 32];
        recipient_arr.copy_from_slice(&recipient);
        
        match client.send(recipient_arr, payload).await {
            Ok(_) => {
                // Update message status to sent
                // TODO: update_message_status(db, &msg.id, "sent")
            }
            Err(_) => {
                // Message will be queued for later
                // TODO: queue.enqueue(recipient_arr, payload)
            }
        }
    }
    
    // Emit event for real-time updates
    let _ = app_handle.emit("message-sent", &msg);
    
    Ok(msg)
}

/// Mark messages from a peer as read.
#[tauri::command]
pub async fn mark_read(
    state: State<'_, AppState>,
    peer_id: String,
) -> Result<(), CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::messages::mark_read(db, &peer_id)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Get total unread count.
#[tauri::command]
pub async fn get_unread_count(
    state: State<'_, AppState>,
) -> Result<i64, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::messages::get_unread_count(db)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}
