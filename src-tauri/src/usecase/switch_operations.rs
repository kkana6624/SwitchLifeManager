use chrono::{DateTime, Utc};

use crate::domain::models::{
    ButtonStats, ControllerProfile, LogicalKey, SwitchData, SwitchHistoryEntry,
};

/// Switch operation service — extracted from `ControllerProfile` methods.
/// Keeps `ControllerProfile` as a pure data structure.
pub struct SwitchOperations;

impl SwitchOperations {
    /// Replace a switch: reset stats, record history, update model ID.
    pub fn replace_switch(
        profile: &mut ControllerProfile,
        key: LogicalKey,
        new_model_id: String,
    ) {
        if let Some(switch) = profile.switches.get_mut(&key) {
            log::info!(
                "AUDIT: ReplaceSwitch for Key: {}. Old Model: {}, Presses: {}, Chatters: {}",
                key,
                switch.switch_model_id,
                switch.stats.total_presses,
                switch.stats.total_chatters
            );

            profile.switch_history.push(SwitchHistoryEntry {
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
            profile.switches.insert(
                key.clone(),
                SwitchData {
                    switch_model_id: new_model_id.clone(),
                    stats: ButtonStats::default(),
                    last_replaced_at: Some(Utc::now()),
                },
            );
        }
    }

    /// Reset stats for a specific switch key: clear stats, record history.
    pub fn reset_stats(profile: &mut ControllerProfile, key: LogicalKey) {
        if let Some(switch) = profile.switches.get_mut(&key) {
            log::info!(
                "AUDIT: ResetStats for Key: {}. Model: {}, Previous Presses: {}, Previous Chatters: {}",
                key,
                switch.switch_model_id,
                switch.stats.total_presses,
                switch.stats.total_chatters
            );

            profile.switch_history.push(SwitchHistoryEntry {
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

    /// Manually set the last replaced date for a switch key.
    pub fn set_last_replaced_date(
        profile: &mut ControllerProfile,
        key: LogicalKey,
        date: DateTime<Utc>,
    ) {
        if let Some(switch) = profile.switches.get_mut(&key) {
            switch.last_replaced_at = Some(date);

            profile.switch_history.push(SwitchHistoryEntry {
                date: Utc::now(),
                key: key.clone(),
                old_model_id: switch.switch_model_id.clone(),
                new_model_id: switch.switch_model_id.clone(),
                previous_stats: switch.stats.clone(),
                event_type: "ManualEdit".to_string(),
            });
        }
    }
}
