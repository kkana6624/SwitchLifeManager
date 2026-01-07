use anyhow::Result;
use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use crossbeam_channel::Receiver;
use log::{error, info};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use crate::domain::models::{
    AppConfig, ButtonStats, LogicalKey, SessionKeyStats, SwitchHistoryEntry, UserProfile,
};
use crate::domain::repositories::SessionRepository;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::infrastructure::timer::HighResolutionTimer;
use crate::usecase::input_monitor::ChatterDetector;

/// Result of the last save operation.
#[derive(Debug, Clone, Serialize)]
pub struct LastSaveResult {
    pub success: bool,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Snapshot of the monitor state for UI consumption.
#[derive(Debug, Clone, Default, Serialize)]
pub struct MonitorSharedState {
    pub is_connected: bool,
    pub is_game_running: bool,
    pub config: AppConfig,

    pub profile_name: String,
    // Use Arc to avoid cloning the map every update
    pub bindings: Arc<HashMap<LogicalKey, u32>>,
    pub switches: HashMap<LogicalKey, crate::domain::models::SwitchData>,
    pub switch_history: Arc<Vec<SwitchHistoryEntry>>,

    // Real-time Input State for Tester
    pub current_pressed_keys: HashSet<LogicalKey>,
    pub raw_button_state: u32,

    pub last_status_message: Option<String>,
    pub last_save_result: Option<LastSaveResult>,

    pub recent_sessions: Vec<crate::domain::models::SessionRecord>,
}

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
        date: DateTime<Utc>,
    },
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
    // Use ArcSwap for lock-free reads from UI thread
    shared_state: Arc<ArcSwap<MonitorSharedState>>,

    // Timer Resolution Control
    high_res_timer: Option<HighResolutionTimer>,

    // Track session start time
    current_session_start: Option<DateTime<Utc>>,

    // Cached Arc for bindings to avoid recreating it when not changed
    cached_bindings: Arc<HashMap<LogicalKey, u32>>,
    cached_history: Arc<Vec<SwitchHistoryEntry>>,

    session_repository: Arc<dyn SessionRepository>,
}

impl<I: InputSource, P: ProcessMonitor, R: ConfigRepository> MonitorService<I, P, R> {
    pub fn new(
        mut input_source: I,
        process_monitor: P,
        repository: R,
        session_repository: Arc<dyn SessionRepository>,
        command_rx: Receiver<MonitorCommand>,
        shared_state: Arc<ArcSwap<MonitorSharedState>>,
    ) -> Result<Self> {
        let profile = repository.load().unwrap_or_else(|e| {
            error!("Failed to load profile, using default: {}", e);
            UserProfile::default()
        });

        // Initialize input method from profile
        input_source.set_input_method(profile.config.input_method.clone());

        let chatter_detector = ChatterDetector::new(profile.config.chatter_threshold_ms);
        let cached_bindings = Arc::new(profile.mapping.bindings.clone());
        let cached_history = Arc::new(profile.switch_history.clone());

        Ok(Self {
            input_source,
            process_monitor,
            repository,
            profile,
            chatter_detector,
            command_rx,
            shared_state,
            high_res_timer: None,
            current_session_start: None,
            cached_bindings,
            cached_history,
            session_repository,
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
                    self.update_status(msg.clone());
                    self.update_save_result(false, msg);
                } else {
                    let msg = "Saved successfully".to_string();
                    self.update_status(msg.clone());
                    self.update_save_result(true, msg);
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
                self.profile.mapping.profile_name = name;
                self.profile.mapping.bindings = bindings;
                // Update cached bindings
                self.cached_bindings = Arc::new(self.profile.mapping.bindings.clone());
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
                    // Set to 0 (unbound) instead of removing the key entirely
                    self.profile.mapping.bindings.insert(old_key.clone(), 0);
                    info!(
                        "Unbound duplicate binding for key: {} (was button {})",
                        old_key, button
                    );
                }

                // Insert new assignment
                self.profile.mapping.bindings.insert(key.clone(), button);
                // Update cached bindings
                self.cached_bindings = Arc::new(self.profile.mapping.bindings.clone());
                info!("Set binding for key: {} -> button {}", key, button);
            }
            MonitorCommand::ReplaceSwitch { key, new_model_id } => {
                if let Some(switch) = self.profile.switches.get_mut(&key) {
                    // Audit Log: Before
                    info!("AUDIT: ReplaceSwitch for Key: {}. Old Model: {}, Presses: {}, Chatters: {}",
                         key, switch.switch_model_id, switch.stats.total_presses, switch.stats.total_chatters);

                    // History
                    self.profile.switch_history.push(SwitchHistoryEntry {
                        date: Utc::now(),
                        key: key.clone(),
                        old_model_id: switch.switch_model_id.clone(),
                        new_model_id: new_model_id.clone(),
                        previous_stats: switch.stats.clone(),
                        event_type: "Replace".to_string(),
                    });
                    self.cached_history = Arc::new(self.profile.switch_history.clone());

                    switch.stats = ButtonStats::default();
                    switch.switch_model_id = new_model_id.clone();
                    switch.last_replaced_at = Some(Utc::now());
                    info!(
                        "Replaced switch for {} with new model {}",
                        key, new_model_id
                    );
                } else {
                    self.profile.switches.insert(
                        key.clone(),
                        crate::domain::models::SwitchData {
                            switch_model_id: new_model_id.clone(),
                            stats: ButtonStats::default(),
                            last_replaced_at: Some(Utc::now()),
                        },
                    );
                    info!("Added new switch for {} with model {}", key, new_model_id);
                }
            }
            MonitorCommand::ResetStats { key } => {
                if let Some(switch) = self.profile.switches.get_mut(&key) {
                    // Audit Log: Before
                    info!("AUDIT: ResetStats for Key: {}. Model: {}, Previous Presses: {}, Previous Chatters: {}",
                         key, switch.switch_model_id, switch.stats.total_presses, switch.stats.total_chatters);

                    // History
                    self.profile.switch_history.push(SwitchHistoryEntry {
                        date: Utc::now(),
                        key: key.clone(),
                        old_model_id: switch.switch_model_id.clone(),
                        new_model_id: switch.switch_model_id.clone(),
                        previous_stats: switch.stats.clone(),
                        event_type: "Reset".to_string(),
                    });
                    self.cached_history = Arc::new(self.profile.switch_history.clone());

                    switch.stats = ButtonStats::default();
                    switch.last_replaced_at = Some(Utc::now());
                    info!("Reset stats for {}", key);
                }
            }
            MonitorCommand::SetLastReplacedDate { key, date } => {
                if let Some(switch) = self.profile.switches.get_mut(&key) {
                    switch.last_replaced_at = Some(date);

                    self.profile.switch_history.push(SwitchHistoryEntry {
                        date: Utc::now(),
                        key: key.clone(),
                        old_model_id: switch.switch_model_id.clone(),
                        new_model_id: switch.switch_model_id.clone(),
                        previous_stats: switch.stats.clone(),
                        event_type: "ManualEdit".to_string(),
                    });
                    self.cached_history = Arc::new(self.profile.switch_history.clone());

                    info!("Set last replaced date for {} to {}", key, date);
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

        let mut last_publish = Instant::now();
        let publish_interval = Duration::from_millis(30); // ~33Hz throttle

        let mut current_pressed_keys = HashSet::new();

        // Main Loop
        'monitor_loop: loop {
            let mut force_publish = false;

            // 1. Process Commands
            while let Ok(cmd) = self.command_rx.try_recv() {
                if let MonitorCommand::Shutdown = cmd {
                    info!("Shutdown command received");
                    break 'monitor_loop;
                }
                self.handle_command(cmd);
                force_publish = true;
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
                         force_publish = true;
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
                    error!("Input Error: {}", e);
                    false
                }
            };

            // State Transition: Connected <-> Disconnected
            if is_connected != was_connected {
                info!(
                    "Connection state changed: {} -> {}",
                    was_connected, is_connected
                );
                was_connected = is_connected;
                force_publish = true;

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

            // 6. Process Monitor (Check Game Status)
            let is_game_running = if last_process_check.elapsed() >= process_check_interval {
                let running = self.check_game_running();
                last_process_check = Instant::now();
                running
            } else {
                was_game_running
            };

            // 5. Process Input
            current_pressed_keys.clear();
            let mut current_raw_buttons = 0;
            if let Ok(w_buttons) = input_result {
                current_raw_buttons = w_buttons;
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                for (key, &mask) in &self.profile.mapping.bindings {
                    let is_pressed = (w_buttons & mask) != 0;
                    if is_pressed {
                        current_pressed_keys.insert(key.clone());
                    }

                    let switch_data =
                        self.profile.switches.entry(key.clone()).or_insert_with(|| {
                            crate::domain::models::SwitchData {
                                switch_model_id: "generic_unknown".to_string(),
                                stats: ButtonStats::default(),
                                last_replaced_at: None,
                            }
                        });

                    // Pass is_game_running (which effectively is the session active flag)
                    self.chatter_detector.process_button(
                        key,
                        is_pressed,
                        now_ms,
                        &mut switch_data.stats,
                        is_game_running,
                    );

                    // Note: session tracking logic is implicitly handled by `process_button` now.
                }
            }

            // Session Logic
            if is_game_running != was_game_running {
                info!(
                    "Game running state changed: {} -> {}",
                    was_game_running, is_game_running
                );
                if is_game_running {
                    // Game Started
                    info!("Game started. Resetting session stats.");
                    self.current_session_start = Some(Utc::now());
                    for switch in self.profile.switches.values_mut() {
                        switch.stats.reset_session_stats();
                    }
                } else {
                    // Game Ended
                    let end_time = Utc::now();
                    info!("Game ended.");

                    if let Some(start_time) = self.current_session_start.take() {
                        let duration_secs = (end_time - start_time).num_seconds().max(0) as u64;

                        let record = crate::domain::models::SessionRecord {
                            id: None,
                            start_time,
                            end_time,
                            duration_secs,
                        };

                        self.profile.recent_sessions.push(record.clone());
                        // Keep last 3 sessions
                        if self.profile.recent_sessions.len() > 3 {
                            self.profile.recent_sessions.remove(0); // Remove oldest
                        }

                        info!("Session recorded: {}s", duration_secs);

                        // --- DB Persistence ---
                        // Convert internal stats to DB stats
                        // We use the 'last_session_*' stats from the switches
                        let mut db_stats = Vec::new();
                        for (key, switch) in &self.profile.switches {
                            // Only record keys that had activity? Or all?
                            // Recording all keys seems safer for complete history.
                            db_stats.push(SessionKeyStats {
                                session_id: 0,             // Ignored by DB insert
                                key_name: key.to_string(), // Use Display impl
                                presses: switch.stats.last_session_presses,
                                chatters: switch.stats.last_session_chatters,
                                chatter_releases: switch.stats.last_session_chatter_releases,
                            });
                        }

                        let repo = self.session_repository.clone();
                        let rec = record.clone();
                        // Blocking call to async DB save
                        tauri::async_runtime::block_on(async move {
                            if let Err(e) = repo.save(&rec, &db_stats).await {
                                error!("Failed to save session to DB: {}", e);
                            } else {
                                info!("Session saved to DB successfully.");
                            }
                        });
                    }

                    // Report logic... (simplified)
                    for (key, switch) in &self.profile.switches {
                        if switch.stats.last_session_presses > 0 {
                            info!("  {}: {} presses", key, switch.stats.last_session_presses);
                        }
                    }
                    // Notify UI if needed (omitted for CLI focus)
                }
                was_game_running = is_game_running;
                force_publish = true;
            }

            // 7. Publish State
            if force_publish || last_publish.elapsed() >= publish_interval {
                self.publish_state(
                    is_connected,
                    is_game_running,
                    &current_pressed_keys,
                    current_raw_buttons,
                );
                last_publish = Instant::now();
            }

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
        self.process_monitor
            .is_process_running(&self.profile.config.target_process_name)
    }

    fn publish_state(
        &self,
        is_connected: bool,
        is_game_running: bool,
        pressed_keys: &HashSet<LogicalKey>,
        raw_buttons: u32,
    ) {
        // Construct new state
        let switches = self.profile.switches.clone();

        // We need to preserve the last status message and save result from the previous state
        // because they might not be updated in this loop iteration.
        // Reading from ArcSwap is cheap.
        let old_state = self.shared_state.load();

        let new_state = MonitorSharedState {
            is_connected,
            is_game_running,
            config: self.profile.config.clone(),
            profile_name: self.profile.mapping.profile_name.clone(),
            bindings: self.cached_bindings.clone(), // Cheap Arc clone
            switches,
            switch_history: self.cached_history.clone(),
            current_pressed_keys: pressed_keys.clone(),
            raw_button_state: raw_buttons,
            last_status_message: old_state.last_status_message.clone(), // Preserve
            last_save_result: old_state.last_save_result.clone(),       // Preserve
            recent_sessions: self.profile.recent_sessions.clone(),
        };

        self.shared_state.store(Arc::new(new_state));
    }

    fn update_status(&self, msg: String) {
        // For simple status updates, we can just publish a new state with the new message.
        // Or we can do a quick load-modify-store loop (CAS not strictly needed if we are the only writer).
        // Since MonitorService is the *only* writer, we can just load and store.
        let old_state = self.shared_state.load();
        let mut new_state = (**old_state).clone();
        new_state.last_status_message = Some(msg);
        self.shared_state.store(Arc::new(new_state));
    }

    fn update_save_result(&self, success: bool, message: String) {
        let old_state = self.shared_state.load();
        let mut new_state = (**old_state).clone();
        new_state.last_save_result = Some(LastSaveResult {
            success,
            message,
            timestamp: Utc::now(),
        });
        self.shared_state.store(Arc::new(new_state));
    }
}
