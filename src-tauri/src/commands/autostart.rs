//! Auto-start commands.

use crate::system::autostart;
use serde::Serialize;

/// Error wrapper for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

/// Check if auto-start is enabled.
#[tauri::command]
pub fn is_autostart_enabled(app_handle: tauri::AppHandle) -> Result<bool, CommandError> {
    Ok(autostart::is_enabled(&app_handle))
}

/// Enable auto-start.
#[tauri::command]
pub fn enable_autostart(app_handle: tauri::AppHandle) -> Result<(), CommandError> {
    autostart::enable(&app_handle).map_err(|e| CommandError { message: e })
}

/// Disable auto-start.
#[tauri::command]
pub fn disable_autostart(app_handle: tauri::AppHandle) -> Result<(), CommandError> {
    autostart::disable(&app_handle).map_err(|e| CommandError { message: e })
}

/// Toggle auto-start.
#[tauri::command]
pub fn toggle_autostart(app_handle: tauri::AppHandle) -> Result<bool, CommandError> {
    let enabled = autostart::is_enabled(&app_handle);
    if enabled {
        autostart::disable(&app_handle).map_err(|e| CommandError { message: e })?;
        Ok(false)
    } else {
        autostart::enable(&app_handle).map_err(|e| CommandError { message: e })?;
        Ok(true)
    }
}
