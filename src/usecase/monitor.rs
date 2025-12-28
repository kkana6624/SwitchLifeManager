use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use crossbeam_channel::Receiver;
use anyhow::Result;
use log::{error, info};

use crate::domain::models::{AppConfig, ButtonStats, LogicalKey, UserProfile};
use crate::domain::interfaces::InputSource;
use crate::domain::errors::InputError;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::infrastructure::timer::HighResolutionTimer;
use crate::usecase::input_monitor::ChatterDetector;

/// Result of the last save operation.
#[derive(Debug, Clone)]
pub struct LastSaveResult {
    pub success: bool,
    pub message: String,
    pub timestamp: SystemTime,
}

/// Snapshot of the monitor state for UI consumption.
#[derive(Debug, Clone, Default)]
pub struct MonitorSharedState {
    pub is_connected: bool,
    pub is_game_running: bool,
    pub target_controller_index: u32,
    pub polling_rate_current: u64,

    pub profile_name: String,
    pub bindings: HashMap<LogicalKey, u16>,
    pub switch_stats: HashMap<LogicalKey, ButtonStats>,
    pub last_status_message: Option<String>,
    pub last_save_result: Option<LastSaveResult>,
}

pub enum MonitorCommand {
    UpdateConfig(AppConfig),
    UpdateMapping(String, HashMap<LogicalKey, u16>), // Preserved for bulk updates (presets)
    SetKeyBinding { key: LogicalKey, button: u16 }, // Added for single key update with conflict resolution
    ReplaceSwitch { key: LogicalKey, new_model_id: String },
    ResetStats { key: LogicalKey },
    Shutdown,
    ForceSave,
}

pub struct MonitorService<I, P, R> {
    input_source: I,
    process_monitor: P,
    repository: R,

    profile: UserProfile,
    chatter_detector: ChatterDetector,

    command_rx: Receiver<MonitorCommand>,
    shared_state: Arc<RwLock<MonitorSharedState>>,

    // Timer Resolution Control
    high_res_timer: Option<HighResolutionTimer>,
}

impl<I: InputSource, P: ProcessMonitor, R: ConfigRepository> MonitorService<I, P, R> {
    pub fn new(
        input_source: I,
        process_monitor: P,
        repository: R,
        command_rx: Receiver<MonitorCommand>,
        shared_state: Arc<RwLock<MonitorSharedState>>,
    ) -> Result<Self> {
        let profile = repository.load().unwrap_or_else(|e| {
            error!("Failed to load profile, using default: {}", e);
            UserProfile::default()
        });

        let chatter_detector = ChatterDetector::new(profile.config.chatter_threshold_ms);

        Ok(Self {
            input_source,
            process_monitor,
            repository,
            profile,
            chatter_detector,
            command_rx,
            shared_state,
            high_res_timer: None,
        })
    }

    fn handle_command(&mut self, cmd: MonitorCommand) {
        match cmd {
            MonitorCommand::Shutdown => {
                info!("Shutdown command received");
            }
            MonitorCommand::ForceSave => {
                if let Err(e) = self.repository.save(&self.profile) {
                    error!("Force save failed: {}", e);
                    let msg = format!("Save failed: {}", e);
                    self.update_status(msg.clone());
                    self.update_save_result(false, msg);
                } else {
                    let msg = "Saved successfully".to_string();
                    self.update_status(msg.clone());
                    self.update_save_result(true, msg);
                }
            }
            MonitorCommand::UpdateConfig(cfg) => {
                self.profile.config = cfg;
                self.chatter_detector = ChatterDetector::new(self.profile.config.chatter_threshold_ms);
                info!("Config updated");
            }
            MonitorCommand::UpdateMapping(name, bindings) => {
                self.profile.mapping.profile_name = name;
                self.profile.mapping.bindings = bindings;
                info!("Mapping updated");
            }
            MonitorCommand::SetKeyBinding { key, button } => {
                // Duplicate Resolution Logic:
                // 1. Check if 'button' is already assigned to any OTHER key.
                // 2. If so, remove that old assignment.
                // 3. Assign 'key' -> 'button'.

                // Identify conflict
                let mut conflict_key = None;
                for (k, &v) in &self.profile.mapping.bindings {
                    if v == button && *k != key {
                        conflict_key = Some(k.clone());
                        break;
                    }
                }

                // Remove old assignment if exists
                if let Some(old_key) = conflict_key {
                    self.profile.mapping.bindings.remove(&old_key);
                    info!("Removed duplicate binding for key: {} (button {})", old_key, button);
                }

                // Insert new assignment
                self.profile.mapping.bindings.insert(key.clone(), button);
                info!("Set binding for key: {} -> button {}", key, button);
            }
            MonitorCommand::ReplaceSwitch { key, new_model_id } => {
                 if let Some(switch) = self.profile.switches.get_mut(&key) {
                     switch.stats = ButtonStats::default();
                     switch.switch_model_id = new_model_id;
                     info!("Replaced switch for {}", key);
                 } else {
                     self.profile.switches.insert(key.clone(), crate::domain::models::SwitchData {
                         switch_model_id: new_model_id,
                         stats: ButtonStats::default(),
                     });
                     info!("Added new switch for {}", key);
                 }
            }
            MonitorCommand::ResetStats { key } => {
                if let Some(switch) = self.profile.switches.get_mut(&key) {
                     switch.stats = ButtonStats::default();
                     info!("Reset stats for {}", key);
                }
            }
        }
    }

    pub fn run(mut self) {
        info!("Monitor Service started");

        let mut last_save_at = Instant::now();
        let save_interval = Duration::from_secs(60);

        let mut was_connected = false;
        let mut was_game_running = false;
        let mut last_process_check = Instant::now();
        let process_check_interval = Duration::from_secs(2);

        // Main Loop
        'monitor_loop: loop {
            // 1. Process Commands
            while let Ok(cmd) = self.command_rx.try_recv() {
                if let MonitorCommand::Shutdown = cmd {
                     info!("Shutdown command received");
                     break 'monitor_loop;
                }
                self.handle_command(cmd);
                self.publish_state(was_connected, was_game_running);
            }

            // 2. Determine Polling Rate
            let target_index = self.profile.config.target_controller_index;
            let polling_rate = if was_connected {
                self.profile.config.polling_rate_ms_connected
            } else {
                self.profile.config.polling_rate_ms_disconnected
            };

            // 3. Wait / Select
            use crossbeam_channel::select;
            select! {
                recv(self.command_rx) -> msg => {
                     if let Ok(cmd) = msg {
                         if let MonitorCommand::Shutdown = cmd {
                             info!("Shutdown command received during wait");
                             break 'monitor_loop;
                         }
                         self.handle_command(cmd);
                         self.publish_state(was_connected, was_game_running);
                         continue 'monitor_loop;
                     }
                },
                default(Duration::from_millis(polling_rate)) => {
                    // Timeout elapsed
                }
            }

            // 4. Input Polling
            let input_result = self.input_source.get_state(target_index);
            let is_connected = match &input_result {
                Ok(_) => true,
                Err(InputError::Disconnected) => false,
                Err(e) => {
                    // Treat other errors as disconnected for safety, but log them
                    // Or keep 'connected' state if it's transient?
                    // "Distinguish disconnect reason... reflect in log/state".
                    // For now, treat as disconnected but specific log.
                    error!("Input Error: {}", e);
                    false
                }
            };

            // State Transition: Connected <-> Disconnected
            if is_connected != was_connected {
                info!("Connection state changed: {} -> {}", was_connected, is_connected);
                was_connected = is_connected;

                if is_connected {
                    // Connected: Enable HighRes Timer
                    if self.high_res_timer.is_none() {
                         self.high_res_timer = Some(HighResolutionTimer::new());
                         info!("High resolution timer enabled");
                    }
                } else {
                    // Disconnected: Disable HighRes Timer
                    if self.high_res_timer.is_some() {
                        self.high_res_timer = None;
                        info!("High resolution timer disabled");
                    }
                }
            }

            // 5. Process Input
            if let Ok(w_buttons) = input_result {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                for (key, &mask) in &self.profile.mapping.bindings {
                    let is_pressed = (w_buttons & mask) != 0;

                    let switch_data = self.profile.switches.entry(key.clone()).or_insert_with(|| {
                         crate::domain::models::SwitchData {
                             switch_model_id: "generic_unknown".to_string(),
                             stats: ButtonStats::default(),
                         }
                    });

                    self.chatter_detector.process_button(key, is_pressed, now_ms, &mut switch_data.stats);

                    // Note: session tracking logic is implicitly handled by `process_button` if we were just incrementing.
                    // But `process_button` only updates total/session stats.
                    // We need to manage session reset on game start.
                }
            }

            // 6. Process Monitor (Check Game Status)
            let is_game_running = if last_process_check.elapsed() >= process_check_interval {
                let running = self.check_game_running();
                last_process_check = Instant::now();
                running
            } else {
                 was_game_running
            };

            // Session Logic
            if is_game_running != was_game_running {
                info!("Game running state changed: {} -> {}", was_game_running, is_game_running);
                if is_game_running {
                    // Game Started: Reset session stats
                    info!("Game started. Resetting session stats.");
                    for switch in self.profile.switches.values_mut() {
                        switch.stats.last_session_presses = 0;
                    }
                } else {
                    // Game Ended: Generate report
                    info!("Game ended. Session Report:");
                    // Ideally we should structure this report better or send event
                    for (key, switch) in &self.profile.switches {
                        if switch.stats.last_session_presses > 0 {
                             info!("  {}: {} presses", key, switch.stats.last_session_presses);
                        }
                    }
                    // Notify UI if needed (omitted for CLI focus)
                }
                was_game_running = is_game_running;
            }

            // 7. Publish State
            self.publish_state(is_connected, is_game_running);

            // 8. Auto Save
            if last_save_at.elapsed() >= save_interval {
                if let Err(e) = self.repository.save(&self.profile) {
                    error!("Auto save failed: {}", e);
                    self.update_save_result(false, format!("Auto save failed: {}", e));
                } else {
                    self.update_save_result(true, "Auto save succeeded".to_string());
                }
                last_save_at = Instant::now();
            }
        }

        // Exit
        info!("Monitor loop exiting. Saving profile...");
        if let Err(e) = self.repository.save(&self.profile) {
            error!("Exit save failed: {}", e);
            self.update_save_result(false, format!("Exit save failed: {}", e));
        } else {
            self.update_save_result(true, "Exit save succeeded".to_string());
        }

        // Ensure RAII timer is dropped (implicit)
        self.high_res_timer = None;
    }

    fn check_game_running(&mut self) -> bool {
        self.process_monitor.is_process_running(&self.profile.config.target_process_name)
    }

    fn publish_state(&self, is_connected: bool, is_game_running: bool) {
        if let Ok(mut state) = self.shared_state.write() {
            state.is_connected = is_connected;
            state.is_game_running = is_game_running;
            state.target_controller_index = self.profile.config.target_controller_index;
            state.polling_rate_current = if is_connected {
                self.profile.config.polling_rate_ms_connected
            } else {
                self.profile.config.polling_rate_ms_disconnected
            };

            state.profile_name = self.profile.mapping.profile_name.clone();
            state.bindings = self.profile.mapping.bindings.clone();

            state.switch_stats.clear();
            for (k, v) in &self.profile.switches {
                state.switch_stats.insert(k.clone(), v.stats.clone());
            }
        }
    }

    fn update_status(&self, msg: String) {
        if let Ok(mut state) = self.shared_state.write() {
            state.last_status_message = Some(msg);
        }
    }

    fn update_save_result(&self, success: bool, message: String) {
        if let Ok(mut state) = self.shared_state.write() {
            state.last_save_result = Some(LastSaveResult {
                success,
                message,
                timestamp: SystemTime::now(),
            });
        }
    }
}
