use anyhow::Result;
use chrono::Utc;
use crossbeam_channel::Receiver;
use log::{error, info};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use crate::domain::models::{AppConfig, LogicalKey, UserProfile};
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::infrastructure::timer::HighResolutionTimer;
use crate::usecase::input_monitor::ChatterDetector;
use crate::usecase::state_publisher::StatePublisher;
use crate::usecase::switch_operations::SwitchOperations;
use crate::usecase::session_manager::SessionManager;

pub enum MonitorCommand {
    UpdateConfig(AppConfig),
    UpdateMapping(String, HashMap<LogicalKey, u32>), // Preserved for bulk updates (presets)
    SetKeyBinding {
        key: LogicalKey,
        button: u32,
    }, // Added for single key update with conflict resolution
    ReplaceSwitch {
        key: LogicalKey,
        new_model_id: String,
    },
    ResetStats {
        key: LogicalKey,
    },
    SetLastReplacedDate {
        key: LogicalKey,
        date: chrono::DateTime<Utc>,
    },
    SetActiveController(String),
    Shutdown,
    ForceSave,
}

pub struct MonitorService<I, P, R> {
    input_source: I,
    process_monitor: P,
    repository: R,

    // Made public for testing
    pub profile: UserProfile,
    chatter_detector: ChatterDetector,

    command_rx: Receiver<MonitorCommand>,
    publisher: StatePublisher,

    // Timer Resolution Control
    high_res_timer: Option<HighResolutionTimer>,

    // Track session start time
    current_session_start: Option<chrono::DateTime<Utc>>,

    // Cached Arc for bindings to avoid recreating it when not changed
    cached_bindings: Arc<HashMap<LogicalKey, u32>>,
}

impl<I: InputSource, P: ProcessMonitor, R: ConfigRepository> MonitorService<I, P, R> {
    pub fn new(
        mut input_source: I,
        process_monitor: P,
        repository: R,
        command_rx: Receiver<MonitorCommand>,
        publisher: StatePublisher,
    ) -> Result<Self> {
        let profile = repository.load().unwrap_or_else(|e| {
            error!("Failed to load profile, using default: {}", e);
            UserProfile::default()
        });

        // Initialize input method from profile
        input_source.set_input_method(profile.config.input_method.clone());

        let chatter_detector = ChatterDetector::new(profile.config.chatter_threshold_ms);
        
        let cached_bindings = {
            let active_profile = profile.controllers.get(&profile.active_controller_id)
                .expect("Active controller profile must exist");
            Arc::new(active_profile.mapping.bindings.clone())
        };

        Ok(Self {
            input_source,
            process_monitor,
            repository,
            profile,
            chatter_detector,
            command_rx,
            publisher,
            high_res_timer: None,
            current_session_start: None,
            cached_bindings,
        })
    }

    // Made public for testing
    pub fn handle_command(&mut self, cmd: MonitorCommand) {
        match cmd {
            MonitorCommand::Shutdown => {
                info!("Shutdown command received");
            }
            MonitorCommand::ForceSave => {
                if let Err(e) = self.repository.save(&self.profile) {
                    error!("Force save failed: {}", e);
                    let msg = format!("Save failed: {}", e);
                    self.publisher.update_status(msg.clone());
                    self.publisher.update_save_result(false, msg);
                } else {
                    let msg = "Saved successfully".to_string();
                    self.publisher.update_status(msg.clone());
                    self.publisher.update_save_result(true, msg);
                }
            }
            MonitorCommand::UpdateConfig(cfg) => {
                // Update input method if changed
                if cfg.input_method != self.profile.config.input_method {
                    self.input_source.set_input_method(cfg.input_method.clone());
                    info!("Input method switched to {:?}", cfg.input_method);
                }

                self.profile.config = cfg;
                self.chatter_detector =
                    ChatterDetector::new(self.profile.config.chatter_threshold_ms);
                info!("Config updated");
            }
            MonitorCommand::UpdateMapping(name, bindings) => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    active_profile.mapping.profile_name = name;
                    active_profile.mapping.bindings = bindings;
                    // Update cached bindings
                    self.cached_bindings = Arc::new(active_profile.mapping.bindings.clone());
                    info!("Mapping updated");
                }
            }
            MonitorCommand::SetKeyBinding { key, button } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    let mut conflict_key = None;
                    for (k, &v) in &active_profile.mapping.bindings {
                        if v == button && *k != key {
                            conflict_key = Some(k.clone());
                            break;
                        }
                    }

                    if let Some(old_key) = conflict_key {
                        active_profile.mapping.bindings.insert(old_key.clone(), 0);
                        info!(
                            "Unbound duplicate binding for key: {} (was button {})",
                            old_key, button
                        );
                    }

                    active_profile.mapping.bindings.insert(key.clone(), button);
                    self.cached_bindings = Arc::new(active_profile.mapping.bindings.clone());
                    info!("Set binding for key: {} -> button {}", key, button);
                }
            }
            MonitorCommand::ReplaceSwitch { key, new_model_id } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    SwitchOperations::replace_switch(active_profile, key.clone(), new_model_id.clone());
                    info!("Replaced switch for {} with new model {}", key, new_model_id);
                }
            }
            MonitorCommand::ResetStats { key } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    SwitchOperations::reset_stats(active_profile, key.clone());
                    info!("Reset stats for {}", key);
                }
            }
            MonitorCommand::SetLastReplacedDate { key, date } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    SwitchOperations::set_last_replaced_date(active_profile, key.clone(), date);
                    info!("Set last replaced date for {} to {}", key, date);
                }
            }
            MonitorCommand::SetActiveController(id) => {
                info!("Setting active controller to: {}", id);
                self.profile.active_controller_id = id.clone();
                // Ensure profile exists
                if !self.profile.controllers.contains_key(&id) {
                    self.profile.controllers.insert(id.clone(), crate::domain::models::ControllerProfile::default());
                }
                let active_profile = self.profile.controllers.get(&id).unwrap();
                self.cached_bindings = Arc::new(active_profile.mapping.bindings.clone());
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

        let mut last_publish = Instant::now();
        let publish_interval = Duration::from_millis(30); // ~33Hz throttle
        
        let mut last_enumerate = Instant::now();
        let enumerate_interval = Duration::from_secs(3);
        let mut connected_controllers = Vec::new();

        let mut current_pressed_keys = HashSet::new();

        // Main Loop
        'monitor_loop: loop {
            let mut force_publish = false;

            // 1. Process Commands
            if !self.process_commands(&mut force_publish) {
                break 'monitor_loop; // Shutdown
            }

            // 2. Determine Polling Rate & Wait
            if !self.wait_for_next_poll(was_connected, &mut force_publish) {
                break 'monitor_loop;
            }

            // 3. Input Polling & Connection State
            let input_result = self.input_source.get_state(self.profile.config.target_controller_index);
            let is_connected = self.handle_connection_state(&input_result, &mut was_connected, &mut force_publish);

            // 4. Process Monitor (Check Game Status)
            let is_game_running = self.check_game_status(&mut last_process_check, process_check_interval, was_game_running);

            // 5. Enumerate controllers
            if last_enumerate.elapsed() >= enumerate_interval {
                if let Ok(controllers) = self.input_source.enumerate_controllers() {
                    connected_controllers = controllers;
                    force_publish = true;
                }
                last_enumerate = Instant::now();
            }

            // 6. Process Input
            let current_raw_buttons = self.process_input(input_result, is_game_running, &mut current_pressed_keys);

            // 7. Session Logic
            self.handle_game_session(is_game_running, &mut was_game_running, &mut force_publish);

            // 8. Publish State
            if force_publish || last_publish.elapsed() >= publish_interval {
                self.publish_current_state(is_connected, is_game_running, &current_pressed_keys, current_raw_buttons, &connected_controllers);
                last_publish = Instant::now();
            }

            // 9. Auto Save
            self.handle_autosave(&mut last_save_at, save_interval);
        }

        self.handle_shutdown();
    }

    fn check_game_running(&mut self) -> bool {
        self.process_monitor
            .is_process_running(&self.profile.config.target_process_name)
    }

    fn process_commands(&mut self, force_publish: &mut bool) -> bool {
        while let Ok(cmd) = self.command_rx.try_recv() {
            if let MonitorCommand::Shutdown = cmd {
                info!("Shutdown command received");
                return false;
            }
            self.handle_command(cmd);
            *force_publish = true;
        }
        true
    }

    fn wait_for_next_poll(&mut self, was_connected: bool, force_publish: &mut bool) -> bool {
        let polling_rate = if was_connected {
            self.profile.config.polling_rate_ms_connected
        } else {
            self.profile.config.polling_rate_ms_disconnected
        };

        use crossbeam_channel::select;
        select! {
            recv(self.command_rx) -> msg => {
                 if let Ok(cmd) = msg {
                     if let MonitorCommand::Shutdown = cmd {
                         info!("Shutdown command received during wait");
                         return false;
                     } else {
                         self.handle_command(cmd);
                         *force_publish = true;
                     }
                 }
            },
            default(Duration::from_millis(polling_rate)) => {
                // Timeout elapsed
            }
        }
        true
    }

    fn handle_connection_state(&mut self, input_result: &Result<u32, InputError>, was_connected: &mut bool, force_publish: &mut bool) -> bool {
        let is_connected = match input_result {
            Ok(_) => true,
            Err(InputError::Disconnected) => false,
            Err(e) => {
                error!("Input Error: {}", e);
                false
            }
        };

        if is_connected != *was_connected {
            info!(
                "Connection state changed: {} -> {}",
                *was_connected, is_connected
            );
            *was_connected = is_connected;
            *force_publish = true;

            if is_connected {
                if self.high_res_timer.is_none() {
                    self.high_res_timer = Some(HighResolutionTimer::new());
                    info!("High resolution timer enabled");
                }
            } else {
                if self.high_res_timer.is_some() {
                    self.high_res_timer = None;
                    info!("High resolution timer disabled");
                }
            }
        }

        is_connected
    }

    fn check_game_status(&mut self, last_process_check: &mut Instant, process_check_interval: Duration, was_game_running: bool) -> bool {
        if last_process_check.elapsed() >= process_check_interval {
            let running = self.check_game_running();
            *last_process_check = Instant::now();
            running
        } else {
            was_game_running
        }
    }

    fn process_input(&mut self, input_result: Result<u32, InputError>, is_game_running: bool, current_pressed_keys: &mut HashSet<LogicalKey>) -> u32 {
        current_pressed_keys.clear();
        let mut current_raw_buttons = 0;

        if let Ok(w_buttons) = input_result {
            current_raw_buttons = w_buttons;
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let active_profile = self.profile.controllers.get_mut(&self.profile.active_controller_id).unwrap();

            for (key, &mask) in &active_profile.mapping.bindings {
                let is_pressed = (w_buttons & mask) != 0;
                if is_pressed {
                    current_pressed_keys.insert(key.clone());
                }

                let switch_data =
                    active_profile.switches.entry(key.clone()).or_insert_with(|| {
                        crate::domain::models::SwitchData {
                            switch_model_id: "generic_unknown".to_string(),
                            stats: crate::domain::models::ButtonStats::default(),
                            last_replaced_at: None,
                        }
                    });

                self.chatter_detector.process_button(
                    key,
                    is_pressed,
                    now_ms,
                    &mut switch_data.stats,
                    is_game_running,
                );
            }
        }
        current_raw_buttons
    }

    fn handle_game_session(&mut self, is_game_running: bool, was_game_running: &mut bool, force_publish: &mut bool) {
        if is_game_running != *was_game_running {
            info!(
                "Game running state changed: {} -> {}",
                *was_game_running, is_game_running
            );
            
            let active_profile = self.profile.controllers.get_mut(&self.profile.active_controller_id).unwrap();

            if is_game_running {
                info!("Game started. Resetting session stats.");
                self.current_session_start = Some(Utc::now());
                SessionManager::start_session(active_profile);
            } else {
                let end_time = Utc::now();
                info!("Game ended.");

                if let Some(start_time) = self.current_session_start.take() {
                    let duration_secs = SessionManager::end_session(active_profile, start_time, end_time);
                    info!("Session recorded: {}s", duration_secs);
                }

                for (key, switch) in &active_profile.switches {
                    if switch.stats.last_session_presses > 0 {
                        info!("  {}: {} presses", key, switch.stats.last_session_presses);
                    }
                }
            }
            *was_game_running = is_game_running;
            *force_publish = true;
        }
    }

    fn publish_current_state(
        &self,
        is_connected: bool,
        is_game_running: bool,
        pressed_keys: &HashSet<LogicalKey>,
        raw_buttons: u32,
        connected_controllers: &[crate::domain::models::ControllerInfo],
    ) {
        let active_profile = self.profile.controllers.get(&self.profile.active_controller_id).unwrap();
        
        self.publisher.publish(
            is_connected,
            is_game_running,
            self.profile.config.clone(),
            active_profile.mapping.profile_name.clone(),
            self.cached_bindings.clone(),
            active_profile.switches.clone(),
            Arc::new(active_profile.switch_history.clone()),
            pressed_keys.clone(),
            raw_buttons,
            active_profile.recent_sessions.clone(),
            self.profile.active_controller_id.clone(),
            connected_controllers.to_vec(),
        );
    }

    fn handle_autosave(&mut self, last_save_at: &mut Instant, save_interval: Duration) {
        if last_save_at.elapsed() >= save_interval {
            if let Err(e) = self.repository.save(&self.profile) {
                error!("Auto save failed: {}", e);
                self.publisher.update_save_result(false, format!("Auto save failed: {}", e));
            } else {
                self.publisher.update_save_result(true, "Auto save succeeded".to_string());
            }
            *last_save_at = Instant::now();
        }
    }

    fn handle_shutdown(&mut self) {
        info!("Monitor loop exiting. Saving profile...");
        if let Err(e) = self.repository.save(&self.profile) {
            error!("Exit save failed: {}", e);
            self.publisher.update_save_result(false, format!("Exit save failed: {}", e));
        } else {
            self.publisher.update_save_result(true, "Exit save succeeded".to_string());
        }
        self.high_res_timer = None;
    }
}
