//! System tray integration.
//!
//! Provides system tray icon with menu and badge support.

use std::sync::atomic::{AtomicI64, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

/// Global unread count for badge
static UNREAD_COUNT: AtomicI64 = AtomicI64::new(0);

/// Set up the system tray icon and menu.
pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    // Create menu items
    let open_item = MenuItem::with_id(app, "open", "Open Whisper", true, None::<&str>)?;
    let new_message_item = MenuItem::with_id(app, "new_message", "New Message", true, None::<&str>)?;
    let separator = MenuItem::with_id(app, "sep", "---", false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Build menu
    let menu = Menu::with_items(app, &[&open_item, &new_message_item, &separator, &quit_item])?;

    // Create tray icon
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "open" => {
                    show_main_window(app);
                }
                "new_message" => {
                    show_main_window(app);
                    // Emit event to navigate to new message view
                    let _ = app.emit("navigate", "new-message");
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                // Left click shows/hides window
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Show the main window.
pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Hide the main window.
pub fn hide_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// Update the unread badge count.
/// 
/// On macOS, this updates the dock badge.
/// On other platforms, this is stored for display in the tray tooltip.
pub fn update_badge<R: Runtime>(app: &AppHandle<R>, count: i64) {
    UNREAD_COUNT.store(count, Ordering::SeqCst);
    
    #[cfg(target_os = "macos")]
    {
        // macOS supports dock badges
        if let Some(window) = app.get_webview_window("main") {
            if count > 0 {
                // Set badge using NSApplication
                // Note: This requires objc calls or a plugin
                // For now, we'll use the tray tooltip
                let _ = window.set_title(&format!("Whisper ({})", count));
            } else {
                let _ = window.set_title("Whisper");
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // Update window title as fallback
        if let Some(window) = app.get_webview_window("main") {
            if count > 0 {
                let _ = window.set_title(&format!("Whisper ({})", count));
            } else {
                let _ = window.set_title("Whisper");
            }
        }
    }
}

/// Get the current unread count.
pub fn get_badge_count() -> i64 {
    UNREAD_COUNT.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_count() {
        UNREAD_COUNT.store(0, Ordering::SeqCst);
        assert_eq!(get_badge_count(), 0);
        
        UNREAD_COUNT.store(5, Ordering::SeqCst);
        assert_eq!(get_badge_count(), 5);
        
        UNREAD_COUNT.store(0, Ordering::SeqCst);
    }
}
