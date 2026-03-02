//! Native notifications.
//!
//! Provides native notification support for incoming messages.

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// Notification click action.
#[derive(Debug, Clone)]
pub struct NotificationAction {
    pub peer_id: String,
}

/// Show a notification for a new message.
pub fn show_message_notification<R: tauri::Runtime>(
    app: &AppHandle<R>,
    sender_name: &str,
    preview: &str,
    peer_id: &str,
) -> Result<(), String> {
    // Truncate preview if too long
    let preview_text = if preview.len() > 100 {
        format!("{}...", &preview[..97])
    } else {
        preview.to_string()
    };
    
    app.notification()
        .builder()
        .title(sender_name)
        .body(&preview_text)
        .action_type_id(peer_id)
        .show()
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Show a generic notification.
pub fn show_notification<R: tauri::Runtime>(
    app: &AppHandle<R>,
    title: &str,
    body: &str,
) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Check if notifications are permitted.
pub fn is_permission_granted<R: tauri::Runtime>(app: &AppHandle<R>) -> bool {
    app.notification()
        .permission_state()
        .map(|state| state == tauri_plugin_notification::PermissionState::Granted)
        .unwrap_or(false)
}

/// Request notification permission.
pub async fn request_permission<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<bool, String> {
    let state = app.notification()
        .request_permission()
        .map_err(|e| e.to_string())?;
    
    Ok(state == tauri_plugin_notification::PermissionState::Granted)
}

#[cfg(test)]
mod tests {
    // Notification tests require runtime context and can't be unit tested easily.
    // Integration tests would use the actual Tauri app.
}
