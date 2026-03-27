mod config;

use config::Config;
use std::{
    process::Command,
    sync::{Arc, RwLock},
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};
use tauri::{
    include_image,
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri::image::Image;

struct AppState {
    config: RwLock<Config>,
    last_brightness: AtomicU32,
}

#[tauri::command]
fn get_brightness(state: State<'_, Arc<AppState>>) -> Result<u32, String> {
    let output = Command::new("ddcutil")
        .args(["getvcp", "10"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout
        .split("current value =")
        .nth(1)
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .ok_or_else(|| format!("Failed to parse: {}", stdout))?;

    state.last_brightness.store(value, Ordering::Relaxed);
    Ok(value)
}

#[tauri::command]
async fn set_brightness(value: u32, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    Command::new("ddcutil")
        .args(["setvcp", "10", &value.to_string()])
        .output()
        .map_err(|e| e.to_string())?;
    state.last_brightness.store(value, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
fn get_config(state: State<'_, Arc<AppState>>) -> Config {
    state.config.read().unwrap().clone()
}

#[tauri::command]
fn save_config(app: AppHandle, new_config: Config, state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let autostart = app.autolaunch();
    if new_config.autostart {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    config::save(&new_config)?;
    *state.config.write().unwrap() = new_config.clone();
    let _ = app.emit("config-updated", new_config);
    Ok(())
}

fn start_resync(state: Arc<AppState>) {
    thread::spawn(move || loop {
        let (enabled, interval_ms) = {
            let cfg = state.config.read().unwrap();
            (cfg.resync_enabled, cfg.resync_interval_ms)
        };

        if enabled {
            let brightness = state.last_brightness.load(Ordering::Relaxed);
            if brightness > 0 {
                let _ = Command::new("ddcutil")
                    .args(["setvcp", "10", &brightness.to_string()])
                    .output();
            }
        }

        thread::sleep(Duration::from_millis(interval_ms.max(500)));
    });
}

fn show_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        if label == "main" {
            if let Some(pos) = popover_position(&window, app) {
                let _ = window.set_position(pos);
            }
        }
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn popover_position(
    window: &tauri::WebviewWindow,
    app: &AppHandle,
) -> Option<tauri::PhysicalPosition<i32>> {
    let win_size = window.outer_size().ok()?;
    let margin = 14i32;

    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten())?;

    let work_area = monitor.work_area();
    let x = work_area.position.x + work_area.size.width as i32 - win_size.width as i32 - margin;
    let y = work_area.position.y + work_area.size.height as i32 - win_size.height as i32 - margin;

    Some(tauri::PhysicalPosition::new(x, y))
}

fn build_tray(app: &AppHandle) -> Result<(), String> {
    let menu = MenuBuilder::new(app)
        .text("show", "Show")
        .text("settings", "Settings")
        .text("quit", "Quit")
        .build()
        .map_err(|e| e.to_string())?;

    TrayIconBuilder::with_id("main-tray")
        .icon(include_image!("icons/tray-icon.png"))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Sun - Brightness")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app, "main"),
            "settings" => show_window(app, "settings"),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(&tray.app_handle(), "main");
            }
        })
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState {
        config: RwLock::new(config::load()),
        last_brightness: AtomicU32::new(0),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            get_brightness,
            set_brightness,
            get_config,
            save_config
        ])
        .setup(move |app| {
            let main = app.get_webview_window("main").unwrap();
            let main_handle = main.clone();
            main.on_window_event(move |event| {
                if let WindowEvent::Focused(false) = event {
                    let _ = main_handle.hide();
                }
            });
            let _ = main.hide();

            let settings = app.get_webview_window("settings").unwrap();
            let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))
                .expect("failed to load icon");
            let _ = settings.set_icon(icon);

            let settings_handle = settings.clone();
            settings.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = settings_handle.hide();
                }
            });
            let _ = settings.hide();

            build_tray(app.handle()).expect("failed to build tray");
            start_resync(state.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}