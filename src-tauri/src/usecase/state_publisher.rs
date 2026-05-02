use arc_swap::ArcSwap;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::domain::models::{AppConfig, ControllerInfo, LogicalKey, SessionRecord, SwitchData, SwitchHistoryEntry};

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
    pub switches: HashMap<LogicalKey, SwitchData>,
    pub switch_history: Arc<Vec<SwitchHistoryEntry>>,

    // Real-time Input State for Tester
    pub current_pressed_keys: HashSet<LogicalKey>,
    pub raw_button_state: u32,

    pub last_status_message: Option<String>,
    pub last_save_result: Option<LastSaveResult>,

    pub recent_sessions: Vec<SessionRecord>,

    pub active_controller_id: String,
    pub connected_controllers: Vec<ControllerInfo>,
}

/// Publisher to handle updating the shared state for the UI
pub struct StatePublisher {
    shared_state: Arc<ArcSwap<MonitorSharedState>>,
}

impl StatePublisher {
    pub fn new(shared_state: Arc<ArcSwap<MonitorSharedState>>) -> Self {
        Self { shared_state }
    }

    pub fn publish(
        &self,
        is_connected: bool,
        is_game_running: bool,
        config: AppConfig,
        profile_name: String,
        bindings: Arc<HashMap<LogicalKey, u32>>,
        switches: HashMap<LogicalKey, SwitchData>,
        switch_history: Arc<Vec<SwitchHistoryEntry>>,
        pressed_keys: HashSet<LogicalKey>,
        raw_buttons: u32,
        recent_sessions: Vec<SessionRecord>,
        active_controller_id: String,
        connected_controllers: Vec<ControllerInfo>,
    ) {
        let old_state = self.shared_state.load();

        let new_state = MonitorSharedState {
            is_connected,
            is_game_running,
            config,
            profile_name,
            bindings,
            switches,
            switch_history,
            current_pressed_keys: pressed_keys,
            raw_button_state: raw_buttons,
            last_status_message: old_state.last_status_message.clone(),
            last_save_result: old_state.last_save_result.clone(),
            recent_sessions,
            active_controller_id,
            connected_controllers,
        };

        self.shared_state.store(Arc::new(new_state));
    }

    pub fn update_status(&self, msg: String) {
        let old_state = self.shared_state.load();
        let mut new_state = (**old_state).clone();
        new_state.last_status_message = Some(msg);
        self.shared_state.store(Arc::new(new_state));
    }

    pub fn update_save_result(&self, success: bool, message: String) {
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
