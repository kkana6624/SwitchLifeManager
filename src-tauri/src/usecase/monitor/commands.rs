use chrono::Utc;
use std::collections::HashMap;

use crate::domain::models::{AppConfig, LogicalKey};

/// Commands that can be sent to the monitor service thread.
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
