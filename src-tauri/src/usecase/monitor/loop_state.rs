use std::collections::HashSet;
use std::time::Instant;

use crate::domain::models::{ControllerInfo, LogicalKey};

/// Consolidated mutable state for the monitor main loop.
/// Previously these were scattered local variables in `run()`.
pub struct MonitorLoopState {
    pub was_connected: bool,
    pub was_game_running: bool,
    pub last_save_at: Instant,
    pub last_process_check: Instant,
    pub last_publish: Instant,
    pub last_enumerate: Instant,
    pub current_pressed_keys: HashSet<LogicalKey>,
    pub connected_controllers: Vec<ControllerInfo>,
}

impl Default for MonitorLoopState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            was_connected: false,
            was_game_running: false,
            last_save_at: now,
            last_process_check: now,
            last_publish: now,
            last_enumerate: now,
            current_pressed_keys: HashSet::new(),
            connected_controllers: Vec::new(),
        }
    }
}
