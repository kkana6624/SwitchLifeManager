use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicalKey {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    E1,
    E2,
    E3,
    E4,
    Other(u16),
}

impl fmt::Display for LogicalKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalKey::Key1 => write!(f, "Key1"),
            LogicalKey::Key2 => write!(f, "Key2"),
            LogicalKey::Key3 => write!(f, "Key3"),
            LogicalKey::Key4 => write!(f, "Key4"),
            LogicalKey::Key5 => write!(f, "Key5"),
            LogicalKey::Key6 => write!(f, "Key6"),
            LogicalKey::Key7 => write!(f, "Key7"),
            LogicalKey::E1 => write!(f, "E1"),
            LogicalKey::E2 => write!(f, "E2"),
            LogicalKey::E3 => write!(f, "E3"),
            LogicalKey::E4 => write!(f, "E4"),
            LogicalKey::Other(id) => write!(f, "Other-{}", id),
        }
    }
}

impl FromStr for LogicalKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Key1" => Ok(LogicalKey::Key1),
            "Key2" => Ok(LogicalKey::Key2),
            "Key3" => Ok(LogicalKey::Key3),
            "Key4" => Ok(LogicalKey::Key4),
            "Key5" => Ok(LogicalKey::Key5),
            "Key6" => Ok(LogicalKey::Key6),
            "Key7" => Ok(LogicalKey::Key7),
            "E1" => Ok(LogicalKey::E1),
            "E2" => Ok(LogicalKey::E2),
            "E3" => Ok(LogicalKey::E3),
            "E4" => Ok(LogicalKey::E4),
            _ => {
                if let Some(rest) = s.strip_prefix("Other-") {
                    let id = rest.parse::<u16>().map_err(|_| format!("Invalid Other ID: {}", rest))?;
                    Ok(LogicalKey::Other(id))
                } else {
                    Err(format!("Unknown LogicalKey: {}", s))
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMethod {
    XInput,
    DirectInput,
}

impl Default for InputMethod {
    fn default() -> Self {
        Self::DirectInput
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub target_controller_index: u32,
    pub input_method: InputMethod,
    pub chatter_threshold_ms: u64,
    pub polling_rate_ms_connected: u64,
    pub polling_rate_ms_disconnected: u64,
    pub target_process_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            target_controller_index: 0,
            input_method: InputMethod::default(),
            chatter_threshold_ms: 15,
            polling_rate_ms_connected: 1,
            polling_rate_ms_disconnected: 1000,
            target_process_name: "bm2dx.exe".to_string(),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonMap {
    pub profile_name: String,
    #[serde_as(as = "HashMap<serde_with::DisplayFromStr, _>")]
    pub bindings: HashMap<LogicalKey, u32>,
}

impl Default for ButtonMap {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        // Default mapping based on user's PhoenixWAN configuration
        bindings.insert(LogicalKey::Key1, 8);
        bindings.insert(LogicalKey::Key2, 1);
        bindings.insert(LogicalKey::Key3, 2);
        bindings.insert(LogicalKey::Key4, 4);
        bindings.insert(LogicalKey::Key5, 64);
        bindings.insert(LogicalKey::Key6, 256);
        bindings.insert(LogicalKey::Key7, 128);
        bindings.insert(LogicalKey::E1, 1024);
        bindings.insert(LogicalKey::E2, 2048);
        bindings.insert(LogicalKey::E3, 8192);
        bindings.insert(LogicalKey::E4, 16384);

        Self {
            profile_name: "Default".to_string(),
            bindings,
        }
    }
}

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
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub schema_version: u32,
    pub config: AppConfig,
    pub mapping: ButtonMap,
    #[serde_as(as = "HashMap<serde_with::DisplayFromStr, _>")]
    pub switches: HashMap<LogicalKey, SwitchData>,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            schema_version: 1,
            config: AppConfig::default(),
            mapping: ButtonMap::default(),
            switches: HashMap::new(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logical_key_serialization() {
        let key = LogicalKey::Key1;
        let s = serde_json::to_string(&key).unwrap();
        assert_eq!(s, "\"Key1\""); // Direct enum serialization default might not match if we rely on map key behavior
    }

    #[test]
    fn test_logical_key_map_serialization() {
        let mut map = HashMap::new();
        map.insert(LogicalKey::Key1, 100);
        map.insert(LogicalKey::Other(12), 200);

        #[serde_as]
        #[derive(Serialize)]
        struct TestStruct {
            #[serde_as(as = "HashMap<serde_with::DisplayFromStr, _>")]
            map: HashMap<LogicalKey, i32>,
        }

        let ts = TestStruct { map };
        let json = serde_json::to_string(&ts).unwrap();

        // Check presence of keys
        assert!(json.contains("\"Key1\":100"));
        assert!(json.contains("\"Other-12\":200"));
    }

    #[test]
    fn test_user_profile_json_structure() {
        let mut profile = UserProfile::default();
        profile.config.target_controller_index = 0;
        profile.mapping.bindings.insert(LogicalKey::Key1, 4096);
        profile.mapping.bindings.insert(LogicalKey::Other(99), 1234);

        let stats = ButtonStats {
            total_presses: 100,
            total_releases: 100,
            total_chatters: 5,
            total_chatter_releases: 5,
            last_session_presses: 10,
            last_session_chatters: 0,
            last_session_chatter_releases: 0,
        };
        profile.switches.insert(LogicalKey::Key1, SwitchData {
            switch_model_id: "omron".to_string(),
            stats,
        });

        let json = serde_json::to_string_pretty(&profile).unwrap();
        println!("{}", json);

        assert!(json.contains("\"Key1\": 4096"));
        assert!(json.contains("\"Other-99\": 1234"));
        assert!(json.contains("\"switch_model_id\": \"omron\""));
        assert!(json.contains("\"target_process_name\": \"bm2dx.exe\""));
    }
}
