#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod domain;
pub mod usecase;
pub mod infrastructure;
pub mod interface;

use std::sync::Arc;
use std::thread;
use crossbeam_channel::bounded;
use simplelog::*;
use log::{info, error};
use arc_swap::ArcSwap;

#[cfg(target_os = "windows")]
use crate::infrastructure::input_source::DynamicInputSource; // Dynamic is preferred
use crate::infrastructure::process_monitor::SysinfoProcessMonitor;
use crate::infrastructure::persistence::FileConfigRepository;
use crate::usecase::monitor::{MonitorService, MonitorCommand, MonitorSharedState};
use crate::domain::models::InputMethod;

fn main() {
    // 1. Setup Logging
    let log_config = ConfigBuilder::new()
        .set_time_offset_to_local()
        .unwrap()
        .build();

    let _ = CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, log_config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Info, log_config, std::fs::File::create("switch_life_manager.log").unwrap()),
    ]);

    info!("SwitchLifeManager Started");

    // 2. Initialize Infrastructure
    // Default to DirectInput (via Gilrs) as per new requirements
    let input_source = DynamicInputSource::new(InputMethod::default());

    let process_monitor = SysinfoProcessMonitor::new();

    // Determine config path
    let repo_path = FileConfigRepository::get_default_config_path().unwrap_or_else(|e| {
        error!("Failed to get default config path: {}, falling back to local 'profile.json'", e);
        std::path::PathBuf::from("profile.json")
    });
    info!("Using config path: {:?}", repo_path);
    let repository = FileConfigRepository::new(repo_path);

    // 3. Initialize Shared State and Channels
    let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));
    let (cmd_tx, cmd_rx) = bounded(10);
    
    // Clone for Ctrl+C handler
    let cmd_tx_sig = cmd_tx.clone();
    ctrlc::set_handler(move || {
        info!("Ctrl+C received! Sending shutdown signal...");
        let _ = cmd_tx_sig.send(MonitorCommand::Shutdown);
    }).expect("Error setting Ctrl-C handler");

    // 4. Create Monitor Service
    let service_result = MonitorService::new(
        input_source,
        process_monitor,
        repository,
        cmd_rx,
        shared_state.clone(),
    );

    match service_result {
        Ok(svc) => {
            // Start Monitor Thread
            let monitor_handle = thread::spawn(move || {
                svc.run();
            });

            info!("Monitor thread started.");

            // 5. Run GUI
            let options = eframe::NativeOptions {
                viewport: eframe::egui::ViewportBuilder::default()
                     .with_inner_size([450.0, 650.0]),
                ..Default::default()
            };

            // Run eframe (blocks main thread)
            let _ = eframe::run_native(
                "SwitchLifeManager",
                options,
                Box::new(|cc| Ok(Box::new(crate::interface::gui::SwitchLifeApp::new(cc, shared_state, cmd_tx))))
            );

            // 6. Cleanup
            // eframe::run_native returns when window is closed.
            // SwitchLifeApp::on_exit sends MonitorCommand::Shutdown.
            
            info!("GUI exited. Waiting for monitor thread...");
            if let Err(e) = monitor_handle.join() {
                error!("Monitor thread panicked: {:?}", e);
            }
            info!("Application exited gracefully.");
        },
        Err(e) => {
            error!("Failed to initialize MonitorService: {}", e);
        }
    }
}