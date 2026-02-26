//! System Tray Module
//! 
//! Handles system tray menu logic and events.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};
use tracing::info;

/// Setup the system tray with menu items
pub fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let start = MenuItem::with_id(app, "start", "Start Engine", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Stop Engine", true, None::<&str>)?;
    let auto_start = MenuItem::with_id(app, "auto_start", "Enable Auto-Start", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &start, &stop, &auto_start, &quit])?;

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Handle tray menu events
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "quit" => {
            info!("Quit requested");
            app.exit(0);
        }
        "show" => {
            info!("Show window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "start" => {
            info!("Start engine");
            // Emit event to start engine
            let _ = app.emit("tray:start-engine", ());
        }
        "stop" => {
            info!("Stop engine");
            // Emit event to stop engine
            let _ = app.emit("tray:stop-engine", ());
        }
        "auto_start" => {
            info!("Toggle auto-start");
            // Emit event to toggle auto-start
            let _ = app.emit("tray:toggle-auto-start", ());
        }
        _ => {}
    }
}
