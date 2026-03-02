//! Relay commands.

use crate::commands::AppState;
use crate::relay::client::{ConnectionState, RelayClient};
use serde::Serialize;
use tauri::{Emitter, State};

/// Error wrapper for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

/// Relay connection status.
#[derive(Debug, Clone, Serialize)]
pub struct RelayStatus {
    pub connected: bool,
    pub state: String,
    pub peer_id: Option<String>,
}

/// Connect to relay server.
#[tauri::command]
pub async fn connect_relay(
    state: State<'_, AppState>,
    url: String,
    app_handle: tauri::AppHandle,
) -> Result<(), CommandError> {
    // Get signing key (clone it to avoid holding lock)
    let signing_key = {
        let guard = state.signing_key.lock().await;
        guard.as_ref().ok_or_else(|| CommandError { 
            message: "Identity not unlocked".into() 
        })?.clone()
    };
    
    // Create relay client
    let client = RelayClient::new(&url, signing_key);
    
    // Connect
    client.connect()
        .await
        .map_err(|e| CommandError { message: e.to_string() })?;
    
    // Store client
    *state.relay.lock().await = Some(client);
    
    // Emit event
    let _ = app_handle.emit("relay-connected", &url);
    
    Ok(())
}

/// Disconnect from relay server.
#[tauri::command]
pub async fn disconnect_relay(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), CommandError> {
    let mut relay = state.relay.lock().await;
    
    if let Some(client) = relay.take() {
        client.disconnect().await;
    }
    
    let _ = app_handle.emit("relay-disconnected", ());
    
    Ok(())
}

/// Get relay connection status.
#[tauri::command]
pub async fn get_relay_status(
    state: State<'_, AppState>,
) -> Result<RelayStatus, CommandError> {
    let relay = state.relay.lock().await;
    
    Ok(match relay.as_ref() {
        Some(client) => {
            let conn_state = client.state().await;
            let state_str = match conn_state {
                ConnectionState::Disconnected => "disconnected",
                ConnectionState::Connecting => "connecting",
                ConnectionState::Authenticating => "authenticating",
                ConnectionState::Connected => "connected",
                ConnectionState::Reconnecting => "reconnecting",
            };
            RelayStatus {
                connected: conn_state == ConnectionState::Connected,
                state: state_str.to_string(),
                peer_id: Some(bs58::encode(client.peer_id()).into_string()),
            }
        }
        None => RelayStatus {
            connected: false,
            state: "disconnected".to_string(),
            peer_id: None,
        },
    })
}
