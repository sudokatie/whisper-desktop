//! Settings commands.

use crate::commands::AppState;
use crate::storage::settings::Settings;
use serde::Serialize;
use tauri::State;

/// Error wrapper for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

/// Get current settings.
#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<Settings, CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::settings::get_settings(db)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Update settings.
#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::settings::update_settings(db, &settings)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}

/// Update a single setting.
#[tauri::command]
pub async fn set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), CommandError> {
    let db = state.db.lock().await;
    let db = db.as_ref().ok_or_else(|| CommandError { message: "Database not initialized".into() })?;
    
    crate::storage::settings::set_setting(db, &key, &value)
        .await
        .map_err(|e| CommandError { message: e.to_string() })
}
