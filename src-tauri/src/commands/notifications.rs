//! Notification commands.

use crate::system::notifications;
use serde::Serialize;

/// Error wrapper for commands
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

/// Check notification permission status.
#[tauri::command]
pub fn check_notification_permission(
    app_handle: tauri::AppHandle,
) -> Result<bool, CommandError> {
    Ok(notifications::is_permission_granted(&app_handle))
}

/// Request notification permission.
#[tauri::command]
pub async fn request_notification_permission(
    app_handle: tauri::AppHandle,
) -> Result<bool, CommandError> {
    notifications::request_permission(&app_handle)
        .await
        .map_err(|e| CommandError { message: e })
}

/// Show a test notification.
#[tauri::command]
pub fn show_test_notification(
    app_handle: tauri::AppHandle,
) -> Result<(), CommandError> {
    notifications::show_notification(
        &app_handle,
        "Whisper",
        "Notifications are working!",
    )
    .map_err(|e| CommandError { message: e })
}

/// Show a message notification.
#[tauri::command]
pub fn show_message_notification(
    app_handle: tauri::AppHandle,
    sender_name: String,
    preview: String,
    peer_id: String,
) -> Result<(), CommandError> {
    notifications::show_message_notification(
        &app_handle,
        &sender_name,
        &preview,
        &peer_id,
    )
    .map_err(|e| CommandError { message: e })
}
