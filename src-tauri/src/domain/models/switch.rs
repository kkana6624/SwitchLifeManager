use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ButtonStats {
    pub total_presses: u64,
    pub total_releases: u64,
    pub total_chatters: u64,
    pub total_chatter_releases: u64,
    
    // Session stats (reset per game session)
    pub last_session_presses: u64,
    pub last_session_chatters: u64,
    pub last_session_chatter_releases: u64,
}

impl ButtonStats {
    pub fn reset_session_stats(&mut self) {
        self.last_session_presses = 0;
        self.last_session_chatters = 0;
        self.last_session_chatter_releases = 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchData {
    pub switch_model_id: String,
    pub stats: ButtonStats,
    #[serde(default)]
    pub last_replaced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchModelInfo {
    pub id: String,
    pub name: String,
    pub manufacturer: String,
    pub rated_lifespan_presses: u64,
}

pub fn get_default_switch_models() -> Vec<SwitchModelInfo> {
    vec![
        SwitchModelInfo {
            id: "omron_d2mv_01_1c3".to_string(),
            name: "D2MV-01-1C3 (50g)".to_string(),
            manufacturer: "Omron".to_string(),
            rated_lifespan_presses: 10_000_000,
        },
        SwitchModelInfo {
            id: "omron_d2mv_01_1c2".to_string(),
            name: "D2MV-01-1C2 (25g)".to_string(),
            manufacturer: "Omron".to_string(),
            rated_lifespan_presses: 10_000_000,
        },
        SwitchModelInfo {
            id: "omron_v_10_1a4".to_string(),
            name: "V-10-1A4 (100g)".to_string(),
            manufacturer: "Omron".to_string(),
            rated_lifespan_presses: 50_000_000,
        },
        SwitchModelInfo {
            id: "generic_unknown".to_string(),
            name: "Generic / Unknown".to_string(),
            manufacturer: "Generic".to_string(),
            rated_lifespan_presses: 1_000_000,
        },
    ]
}
