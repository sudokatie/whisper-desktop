//! Launch on startup.
//!
//! Provides auto-start functionality using Tauri's autostart plugin.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// Check if auto-start is enabled.
pub fn is_enabled<R: tauri::Runtime>(app: &AppHandle<R>) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

/// Enable auto-start.
pub fn enable<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    app.autolaunch()
        .enable()
        .map_err(|e| e.to_string())
}

/// Disable auto-start.
pub fn disable<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    app.autolaunch()
        .disable()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    // Auto-start tests require system integration and can't be unit tested easily.
    // These would be tested manually or via integration tests.
}
