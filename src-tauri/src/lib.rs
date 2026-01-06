pub mod app_state;
pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod usecase;

use arc_swap::ArcSwap;
use crossbeam_channel::unbounded;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

use crate::app_state::AppState;
use crate::domain::models::InputMethod;
use crate::infrastructure::input_source::DynamicInputSource;
use crate::infrastructure::persistence::{ConfigRepository, FileConfigRepository};
use crate::infrastructure::process_monitor::SysinfoProcessMonitor;
use crate::usecase::monitor::{MonitorService, MonitorSharedState};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::get_snapshot,
            commands::force_save,
            commands::set_binding,
            commands::reset_stats,
            commands::replace_switch,
            commands::update_config,
            commands::set_target_controller,
            commands::reset_to_default_mapping,
            commands::set_last_replaced_date,
            commands::set_obs_enabled,
            commands::set_obs_port,
            commands::set_obs_poll_interval,
            commands::get_obs_status,
        ])
        .setup(|app| {
            // --- Logger Setup ---
            if let Ok(config_path) = FileConfigRepository::get_default_config_path() {
                let log_path = config_path.with_file_name("app.log");
                if let Some(parent) = log_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                let _ = simplelog::WriteLogger::init(
                    simplelog::LevelFilter::Info,
                    simplelog::Config::default(),
                    std::fs::File::create(log_path).unwrap_or_else(|_| {
                        std::fs::File::create("switch_life_manager.log").unwrap()
                    }),
                );
            }

            // --- Monitor Setup ---
            let (command_tx, command_rx) = unbounded();
            let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

            // Repositories & Services
            // Default config path: %LOCALAPPDATA%/SwitchLifeManager/profile.json
            let config_path = FileConfigRepository::get_default_config_path()
                .expect("Failed to determine config path");
            let repository = FileConfigRepository::new(config_path);
            let input_source = DynamicInputSource::new(InputMethod::default());
            let process_monitor = SysinfoProcessMonitor::new();

            // --- OBS Server Setup ---
            let obs_server = Arc::new(crate::infrastructure::obs_server::ObsServer::new());

            // Auto-start check
            // We load the profile here just to check the config. MonitorService will load it again, which is fine (read-only).
            // Note: MonitorService owns the "write" authority for profile, so this is just a snapshot read.
            if let Ok(profile) = repository.load() {
                if profile.config.obs_enabled {
                    let obs = obs_server.clone();
                    let state = shared_state.clone();
                    let port = profile.config.obs_port;
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = obs.start(port, state).await {
                            log::error!("Failed to auto-start OBS server: {}", e);
                        }
                    });
                }
            }

            // Spawn Monitor Thread
            let service_shared_state = shared_state.clone();
            thread::spawn(move || {
                let service = MonitorService::new(
                    input_source,
                    process_monitor,
                    repository,
                    command_rx,
                    service_shared_state,
                )
                .expect("Failed to create MonitorService");
                service.run();
            });

            // Manage App State
            app.manage(AppState::new(shared_state.clone(), command_tx, obs_server));

            // --- State Emit Loop ---
            let app_handle = app.handle().clone();
            let emit_shared_state = shared_state.clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(33)); // ~30fps
                loop {
                    interval.tick().await;
                    let state = emit_shared_state.load();
                    // Emit state-update event to frontend
                    if let Err(e) = app_handle.emit("state-update", &**state) {
                        // This might fail if the app is shutting down, which is fine
                        log::trace!("Failed to emit state-update: {}", e);
                    }
                }
            });

            // --- Tray Icon Setup ---
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
