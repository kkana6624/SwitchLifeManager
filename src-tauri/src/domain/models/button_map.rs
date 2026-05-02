use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;

use super::LogicalKey;

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
