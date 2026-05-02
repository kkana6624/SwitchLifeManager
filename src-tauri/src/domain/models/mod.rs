mod logical_key;
mod config;
mod button_map;
mod switch;
mod session;
mod profile;

// Re-export all public types so that `use crate::domain::models::*` continues to work.
pub use logical_key::LogicalKey;
pub use config::{InputMethod, AppConfig};
pub use button_map::ButtonMap;
pub use switch::{ButtonStats, SwitchData, SwitchModelInfo, get_default_switch_models};
pub use session::{SwitchHistoryEntry, SessionRecord, SessionKeyStats};
pub use profile::{ControllerInfo, ControllerProfile, UserProfile};

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_with::serde_as;
    use std::collections::HashMap;

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
        let controller_profile = profile.controllers.get_mut("default").unwrap();
        controller_profile.mapping.bindings.insert(LogicalKey::Key1, 4096);
        controller_profile.mapping.bindings.insert(LogicalKey::Other(99), 1234);

        let stats = ButtonStats {
            total_presses: 100,
            total_releases: 100,
            total_chatters: 5,
            total_chatter_releases: 5,
            last_session_presses: 10,
            last_session_chatters: 0,
            last_session_chatter_releases: 0,
        };
        controller_profile.switches.insert(LogicalKey::Key1, SwitchData {
            switch_model_id: "omron".to_string(),
            stats,
            last_replaced_at: None,
        });

        let json = serde_json::to_string_pretty(&profile).unwrap();
        println!("{}", json);

        assert!(json.contains("\"Key1\": 4096"));
        assert!(json.contains("\"Other-99\": 1234"));
        assert!(json.contains("\"switch_model_id\": \"omron\""));
        assert!(json.contains("\"target_process_name\": \"bm2dx.exe\""));
    }
}
