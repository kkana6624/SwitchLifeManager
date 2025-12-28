pub mod domain;
pub mod usecase;
pub mod infrastructure;
pub mod interface;

use std::sync::{Arc, RwLock};
use std::thread;
use crossbeam_channel::bounded;
use simplelog::*;
use log::{info, error};

#[cfg(target_os = "windows")]
use crate::infrastructure::input_source::XInputSource;
use crate::infrastructure::process_monitor::SysinfoProcessMonitor;
use crate::infrastructure::persistence::FileConfigRepository;
use crate::usecase::monitor::{MonitorService, MonitorCommand, MonitorSharedState};

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

    info!("SwitchLifeManager CLI Started");

    // 2. Initialize Infrastructure
    #[cfg(target_os = "windows")]
    let input_source = XInputSource::new();
    #[cfg(not(target_os = "windows"))]
    let input_source = crate::infrastructure::input_source::MockInputSource::new(vec![]); // Should probably provide meaningful mock or error

    let process_monitor = SysinfoProcessMonitor::new();

    // Determine config path (local directory for CLI simplicity)
    let repo_path = std::path::PathBuf::from("profile.json");
    let repository = FileConfigRepository::new(repo_path);

    // 3. Initialize Shared State and Channels
    let shared_state = Arc::new(RwLock::new(MonitorSharedState::default()));
    let (cmd_tx, cmd_rx) = bounded(10);
    let cmd_tx_clone = cmd_tx.clone();

    // 4. Setup Signal Handling (Ctrl+C)
    ctrlc::set_handler(move || {
        info!("Ctrl+C received! Sending shutdown signal...");
        let _ = cmd_tx_clone.send(MonitorCommand::Shutdown);
    }).expect("Error setting Ctrl-C handler");

    // 5. Create and Start Monitor Service
    let service = MonitorService::new(
        input_source,
        process_monitor,
        repository,
        cmd_rx,
        shared_state.clone(),
    );

    match service {
        Ok(svc) => {
            let monitor_handle = thread::spawn(move || {
                svc.run();
            });

            info!("Monitor thread running. Press Ctrl+C to exit.");

            // Wait for monitor thread to finish
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
