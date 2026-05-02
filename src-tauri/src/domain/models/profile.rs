use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

use super::{
    AppConfig, ButtonMap, SwitchData, SwitchHistoryEntry, SessionRecord, LogicalKey,
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
