use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::LogicalKey;
use super::ButtonStats;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchHistoryEntry {
    pub date: DateTime<Utc>,
    pub key: LogicalKey,
    pub old_model_id: String,
    pub new_model_id: String,
    pub previous_stats: ButtonStats,
    pub event_type: String, // "Replace", "Reset", "ManualEdit"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_secs: u64,
    #[serde(default)]
    pub stats: HashMap<LogicalKey, SessionKeyStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKeyStats {
    pub presses: u64,
    pub chatters: u64,
}
