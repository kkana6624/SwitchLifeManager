//! Monitor service — the main loop that coordinates input reading,
//! chatter detection, session management, and state publishing.
//!
//! # Module structure
//!
//! - `commands` — `MonitorCommand` enum
//! - `command_handler` — command dispatch logic
//! - `input_processing` — input polling, connection state, game status
//! - `game_session` — game session start/end lifecycle
//! - `publishing` — state publishing, autosave, shutdown
//! - `loop_state` — `MonitorLoopState` consolidating loop variables

mod command_handler;
pub mod commands;
mod game_session;
mod input_processing;
mod loop_state;
mod publishing;

// Re-export public API so that `use crate::usecase::monitor::{MonitorService, MonitorCommand}` works.
pub use commands::MonitorCommand;

use anyhow::Result;
use crossbeam_channel::Receiver;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::domain::interfaces::InputSource;
use crate::domain::models::{LogicalKey, UserProfile};
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::infrastructure::timer::HighResolutionTimer;
use crate::usecase::input_monitor::ChatterDetector;
use crate::usecase::state_publisher::StatePublisher;

use loop_state::MonitorLoopState;

pub struct MonitorService<I, P, R> {
    pub(crate) input_source: I,
    pub(crate) process_monitor: P,
    pub(crate) repository: R,

    // Made public for testing
    pub profile: UserProfile,
    pub(crate) chatter_detector: ChatterDetector,

    pub(crate) command_rx: Receiver<MonitorCommand>,
    pub(crate) publisher: StatePublisher,

    // Timer Resolution Control
    pub(crate) high_res_timer: Option<HighResolutionTimer>,

    // Track session start time
    pub(crate) current_session_start: Option<chrono::DateTime<chrono::Utc>>,

    // Cached Arc for bindings to avoid recreating it when not changed
    pub(crate) cached_bindings: Arc<HashMap<LogicalKey, u32>>,
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

    pub fn run(mut self) {
        info!("Monitor Service started");

        let save_interval = Duration::from_secs(60);
        let process_check_interval = Duration::from_secs(2);
        let publish_interval = Duration::from_millis(30); // ~33Hz throttle
        let enumerate_interval = Duration::from_secs(3);

        let mut state = MonitorLoopState::default();

        // Main Loop
        'monitor_loop: loop {
            let mut force_publish = false;

            // 1. Process Commands
            if !self.process_commands(&mut force_publish) {
                break 'monitor_loop;
            }

            // 2. Determine Polling Rate & Wait
            if !self.wait_for_next_poll(state.was_connected, &mut force_publish) {
                break 'monitor_loop;
            }

            // 3. Input Polling & Connection State
            let input_result = self.input_source.get_state(self.profile.config.target_controller_index);
            let is_connected = self.handle_connection_state(&input_result, &mut state.was_connected, &mut force_publish);

            // 4. Process Monitor (Check Game Status)
            let is_game_running = self.check_game_status(&mut state.last_process_check, process_check_interval, state.was_game_running);

            // 5. Enumerate controllers
            if state.last_enumerate.elapsed() >= enumerate_interval {
                if let Ok(controllers) = self.input_source.enumerate_controllers() {
                    state.connected_controllers = controllers;
                    force_publish = true;
                }
                state.last_enumerate = std::time::Instant::now();
            }

            // 6. Process Input
            let current_raw_buttons = self.process_input(input_result, is_game_running, &mut state.current_pressed_keys);

            // 7. Session Logic
            self.handle_game_session(is_game_running, &mut state.was_game_running, &mut force_publish);

            // 8. Publish State
            if force_publish || state.last_publish.elapsed() >= publish_interval {
                self.publish_current_state(is_connected, is_game_running, &state.current_pressed_keys, current_raw_buttons, &state.connected_controllers);
                state.last_publish = std::time::Instant::now();
            }

            // 9. Auto Save
            self.handle_autosave(&mut state.last_save_at, save_interval);
        }

        self.handle_shutdown();
    }
}
