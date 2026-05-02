use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::domain::models::{ControllerProfile, SessionKeyStats, SessionRecord};

/// Session management service — extracted from `ControllerProfile` methods.
/// Handles game session start/end logic.
pub struct SessionManager;

impl SessionManager {
    /// Start a new session: reset per-session stats for all switches.
    pub fn start_session(profile: &mut ControllerProfile) {
        for switch in profile.switches.values_mut() {
            switch.stats.reset_session_stats();
        }
    }

    /// End a session: collect session stats, create a record, and store it.
    /// Returns the duration in seconds.
    pub fn end_session(
        profile: &mut ControllerProfile,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> u64 {
        let duration_secs = (end_time - start_time).num_seconds().max(0) as u64;

        let mut stats = HashMap::new();
        for (key, switch) in &profile.switches {
            if switch.stats.last_session_presses > 0 || switch.stats.last_session_chatters > 0 {
                stats.insert(
                    key.clone(),
                    SessionKeyStats {
                        presses: switch.stats.last_session_presses,
                        chatters: switch.stats.last_session_chatters,
                    },
                );
            }
        }

        let record = SessionRecord {
            start_time,
            end_time,
            duration_secs,
            stats,
        };

        profile.recent_sessions.push(record);
        if profile.recent_sessions.len() > 10 {
            // Increased to 10 for better history
            profile.recent_sessions.remove(0);
        }

        duration_secs
    }
}
