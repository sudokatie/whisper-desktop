//! Whisper Desktop - End-to-end encrypted messaging
//!
//! This is the Tauri backend for the Whisper Desktop application.

pub mod crypto;
pub mod storage;
pub mod relay;
pub mod sync;
pub mod system;
pub mod commands;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState::default())
        .setup(|app| {
            // Initialize logging
            tracing_subscriber::fmt()
                .with_env_filter("whisper_desktop=debug")
                .init();

            tracing::info!("Whisper Desktop starting...");
            
            // Get app data directory
            let app_data = app.path().app_data_dir()
                .expect("Failed to get app data directory");
            
            // Create directory if needed
            std::fs::create_dir_all(&app_data).ok();
            
            let db_path = app_data.join("whisper.db");
            let state = app.state::<AppState>();
            
            // Initialize database in background
            let db_state = state.db.clone();
            tauri::async_runtime::spawn(async move {
                match storage::database::Database::open(&db_path).await {
                    Ok(db) => {
                        *db_state.lock().await = Some(db);
                        tracing::info!("Database initialized");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize database: {}", e);
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Identity commands
            commands::identity::create_identity,
            commands::identity::get_identity,
            commands::identity::unlock,
            commands::identity::lock,
            commands::identity::change_passphrase,
            commands::identity::get_link_qr,
            // Message commands
            commands::messages::get_conversations,
            commands::messages::get_messages,
            commands::messages::send_message,
            commands::messages::mark_read,
            commands::messages::get_unread_count,
            // Contact commands
            commands::contacts::get_contacts,
            commands::contacts::add_contact,
            commands::contacts::add_contact_from_qr,
            commands::contacts::update_contact_alias,
            commands::contacts::update_contact_trust,
            commands::contacts::delete_contact,
            // Settings commands
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::set_setting,
            // Relay commands
            commands::relay::connect_relay,
            commands::relay::disconnect_relay,
            commands::relay::get_relay_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
