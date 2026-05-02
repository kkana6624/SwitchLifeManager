use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::{
    LogicalKey, AppConfig, ButtonMap,
    ButtonStats, SwitchData,
    SwitchHistoryEntry, SessionRecord, SessionKeyStats,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControllerInfo {
    pub id: String,
    pub name: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerProfile {
    pub mapping: ButtonMap,
    #[serde_as(as = "HashMap<serde_with::DisplayFromStr, _>")]
    pub switches: HashMap<LogicalKey, SwitchData>,
    #[serde(default)]
    pub switch_history: Vec<SwitchHistoryEntry>,
    #[serde(default)]
    pub recent_sessions: Vec<SessionRecord>,
}

impl Default for ControllerProfile {
    fn default() -> Self {
        Self {
            mapping: ButtonMap::default(),
            switches: HashMap::new(),
            switch_history: Vec::new(),
            recent_sessions: Vec::new(),
        }
    }
}

impl ControllerProfile {
    pub fn replace_switch(&mut self, key: LogicalKey, new_model_id: String) {
        if let Some(switch) = self.switches.get_mut(&key) {
            log::info!("AUDIT: ReplaceSwitch for Key: {}. Old Model: {}, Presses: {}, Chatters: {}",
                 key, switch.switch_model_id, switch.stats.total_presses, switch.stats.total_chatters);

            self.switch_history.push(SwitchHistoryEntry {
                date: Utc::now(),
                key: key.clone(),
                old_model_id: switch.switch_model_id.clone(),
                new_model_id: new_model_id.clone(),
                previous_stats: switch.stats.clone(),
                event_type: "Replace".to_string(),
            });

            switch.stats = ButtonStats::default();
            switch.switch_model_id = new_model_id.clone();
            switch.last_replaced_at = Some(Utc::now());
        } else {
            self.switches.insert(
                key.clone(),
                SwitchData {
                    switch_model_id: new_model_id.clone(),
                    stats: ButtonStats::default(),
                    last_replaced_at: Some(Utc::now()),
                },
            );
        }
    }

    pub fn reset_switch_stats(&mut self, key: LogicalKey) {
        if let Some(switch) = self.switches.get_mut(&key) {
            log::info!("AUDIT: ResetStats for Key: {}. Model: {}, Previous Presses: {}, Previous Chatters: {}",
                 key, switch.switch_model_id, switch.stats.total_presses, switch.stats.total_chatters);

            self.switch_history.push(SwitchHistoryEntry {
                date: Utc::now(),
                key: key.clone(),
                old_model_id: switch.switch_model_id.clone(),
                new_model_id: switch.switch_model_id.clone(),
                previous_stats: switch.stats.clone(),
                event_type: "Reset".to_string(),
            });

            switch.stats = ButtonStats::default();
            switch.last_replaced_at = Some(Utc::now());
        }
    }

    pub fn set_last_replaced_date(&mut self, key: LogicalKey, date: DateTime<Utc>) {
        if let Some(switch) = self.switches.get_mut(&key) {
            switch.last_replaced_at = Some(date);

            self.switch_history.push(SwitchHistoryEntry {
                date: Utc::now(),
                key: key.clone(),
                old_model_id: switch.switch_model_id.clone(),
                new_model_id: switch.switch_model_id.clone(),
                previous_stats: switch.stats.clone(),
                event_type: "ManualEdit".to_string(),
            });
        }
    }

    pub fn start_session(&mut self) {
        for switch in self.switches.values_mut() {
            switch.stats.reset_session_stats();
        }
    }

    pub fn end_session(&mut self, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> u64 {
        let duration_secs = (end_time - start_time).num_seconds().max(0) as u64;

        let mut stats = HashMap::new();
        for (key, switch) in &self.switches {
            if switch.stats.last_session_presses > 0 || switch.stats.last_session_chatters > 0 {
                stats.insert(key.clone(), SessionKeyStats {
                    presses: switch.stats.last_session_presses,
                    chatters: switch.stats.last_session_chatters,
                });
            }
        }

        let record = SessionRecord {
            start_time,
            end_time,
            duration_secs,
            stats,
        };

        self.recent_sessions.push(record);
        if self.recent_sessions.len() > 10 { // Increased to 10 for better history
            self.recent_sessions.remove(0);
        }

        duration_secs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub schema_version: u32,
    pub config: AppConfig,
    #[serde(default)]
    pub active_controller_id: String,
    #[serde(default)]
    pub controllers: HashMap<String, ControllerProfile>,
}

impl Default for UserProfile {
    fn default() -> Self {
        let default_id = "default".to_string();
        let mut controllers = HashMap::new();
        controllers.insert(default_id.clone(), ControllerProfile::default());

        Self {
            schema_version: 2,
            config: AppConfig::default(),
            active_controller_id: default_id,
            controllers,
        }
    }
}
