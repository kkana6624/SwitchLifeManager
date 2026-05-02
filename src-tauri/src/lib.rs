pub mod app_state;
pub mod commands;
pub mod domain;
pub mod infrastructure;
pub mod usecase;
mod logging;
mod tray;

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use arc_swap::ArcSwap;
use crossbeam_channel::unbounded;
use tauri::{Emitter, Manager, WindowEvent};

use crate::app_state::AppState;
use crate::domain::models::InputMethod;
use crate::infrastructure::input_source::DynamicInputSource;
use crate::infrastructure::persistence::FileConfigRepository;
use crate::infrastructure::process_monitor::SysinfoProcessMonitor;
use crate::usecase::monitor::MonitorService;
use crate::usecase::state_publisher::{MonitorSharedState, StatePublisher};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::force_save,
            commands::set_binding,
            commands::reset_stats,
            commands::replace_switch,
            commands::update_config,
            commands::set_target_controller,
            commands::reset_to_default_mapping,
            commands::set_last_replaced_date,
            commands::set_active_controller
        ])
        .setup(|app| {
            logging::init_logger();

            let (command_tx, command_rx) = unbounded();
            let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

            // Spawn Monitor Service thread
            setup_monitor(command_rx, shared_state.clone());
            app.manage(AppState::new(shared_state.clone(), command_tx));

            // Start frontend state emit loop
            start_emit_loop(app.handle().clone(), shared_state);

            // Setup system tray
            tray::setup_tray(app)?;

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

/// Create and spawn the monitor service on a background thread.
fn setup_monitor(
    command_rx: crossbeam_channel::Receiver<crate::usecase::monitor::MonitorCommand>,
    shared_state: Arc<ArcSwap<MonitorSharedState>>,
) {
    let config_path = FileConfigRepository::get_default_config_path()
        .expect("Failed to determine config path");
    let repository = FileConfigRepository::new(config_path);
    let input_source = DynamicInputSource::new(InputMethod::default());
    let process_monitor = SysinfoProcessMonitor::new();

    thread::spawn(move || {
        let publisher = StatePublisher::new(shared_state);
        let service = MonitorService::new(
            input_source,
            process_monitor,
            repository,
            command_rx,
            publisher,
        )
        .expect("Failed to create MonitorService");
        service.run();
    });
}

/// Spawn an async loop that emits state updates to the frontend at ~30fps.
fn start_emit_loop(
    app_handle: tauri::AppHandle,
    shared_state: Arc<ArcSwap<MonitorSharedState>>,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(33));
        loop {
            interval.tick().await;
            let state = shared_state.load();
            if let Err(e) = app_handle.emit("state-update", &**state) {
                log::trace!("Failed to emit state-update: {}", e);
            }
        }
    });
}
