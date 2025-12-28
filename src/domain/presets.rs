use std::collections::HashMap;
use crate::domain::models::{ButtonMap, LogicalKey};

/// Generates a ButtonMap for the Official Controller.
///
/// Assumptions based on standard layouts:
/// Buttons 1-7 map to Keys 1-7.
/// E1, E2, Start, Select map to higher buttons.
/// Turntable is provisionally mapped to a button (e.g., Button 8/16).
pub fn get_official_controller_map() -> ButtonMap {
    let mut bindings = HashMap::new();

    // Standard mapping assumption
    // Bit 0 = 1 << 0 = 1
    bindings.insert(LogicalKey::Key1, 1 << 0);
    bindings.insert(LogicalKey::Key2, 1 << 1);
    bindings.insert(LogicalKey::Key3, 1 << 2);
    bindings.insert(LogicalKey::Key4, 1 << 3);
    bindings.insert(LogicalKey::Key5, 1 << 4);
    bindings.insert(LogicalKey::Key6, 1 << 5);
    bindings.insert(LogicalKey::Key7, 1 << 6);

    // E1/E2/Start/Select
    // Assuming:
    // E1 = Button 9 (1 << 8)
    // E2 = Button 10 (1 << 9)
    // Start = Button 11 (1 << 10)
    // Select = Button 12 (1 << 11)
    bindings.insert(LogicalKey::E1, 1 << 8);
    bindings.insert(LogicalKey::E2, 1 << 9);
    bindings.insert(LogicalKey::Start, 1 << 10);
    bindings.insert(LogicalKey::Select, 1 << 11);

    // Turntable (Provisional)
    // Often mapped to Button 8? Or Axis?
    // Task says "Turntable treated as button (bitmask)".
    // Let's use Button 8 (1 << 7)
    bindings.insert(LogicalKey::Turntable, 1 << 7);

    ButtonMap {
        profile_name: "Official Controller".to_string(),
        bindings,
    }
}

/// Generates a ButtonMap for the PhoenixWAN Controller.
///
/// PhoenixWAN often has similar mappings but might differ.
/// For this implementation, we will use a compatible mapping
/// but ensure it's distinguished by name.
pub fn get_phoenix_wan_map() -> ButtonMap {
    // PhoenixWAN often follows the Konami standard for button numbers,
    // but might have extra buttons or different TT mapping.
    // For now, we replicate the standard layout as a safe default.
    let mut map = get_official_controller_map();
    map.profile_name = "PhoenixWAN".to_string();
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_official_controller_map_contents() {
        let map = get_official_controller_map();
        assert_eq!(map.profile_name, "Official Controller");

        // Check core keys
        assert_eq!(map.bindings.get(&LogicalKey::Key1), Some(&1));
        assert_eq!(map.bindings.get(&LogicalKey::Key7), Some(&64));
        assert_eq!(map.bindings.get(&LogicalKey::Start), Some(&1024));

        // Ensure all required keys are present
        assert!(map.bindings.contains_key(&LogicalKey::Key1));
        assert!(map.bindings.contains_key(&LogicalKey::Key2));
        assert!(map.bindings.contains_key(&LogicalKey::Key3));
        assert!(map.bindings.contains_key(&LogicalKey::Key4));
        assert!(map.bindings.contains_key(&LogicalKey::Key5));
        assert!(map.bindings.contains_key(&LogicalKey::Key6));
        assert!(map.bindings.contains_key(&LogicalKey::Key7));
        assert!(map.bindings.contains_key(&LogicalKey::Turntable));
        assert!(map.bindings.contains_key(&LogicalKey::Start));
        assert!(map.bindings.contains_key(&LogicalKey::Select));
    }

    #[test]
    fn test_phoenix_wan_map_contents() {
        let map = get_phoenix_wan_map();
        assert_eq!(map.profile_name, "PhoenixWAN");
        // Inherits bindings
        assert_eq!(map.bindings.get(&LogicalKey::Key1), Some(&1));
    }
}
