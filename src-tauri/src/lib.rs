//! Whisper Desktop - End-to-end encrypted messaging
//!
//! This is the Tauri backend for the Whisper Desktop application.

pub mod crypto;
pub mod storage;
pub mod relay;
pub mod sync;
pub mod system;
pub mod commands;

use tauri::Manager;

/// Temporary greet command for testing
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Whisper Desktop.", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize logging
            tracing_subscriber::fmt()
                .with_env_filter("whisper_desktop=debug")
                .init();

            tracing::info!("Whisper Desktop starting...");
            
            // Future: Initialize storage, relay connection, system tray
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // commands::identity::create_identity,
            // commands::identity::unlock,
            // commands::identity::lock,
            // commands::messages::get_conversations,
            // commands::messages::get_messages,
            // commands::messages::send_message,
            // commands::contacts::get_contacts,
            // commands::contacts::add_contact,
            // commands::settings::get_settings,
            // commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
