#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use std::thread;
    use arc_swap::ArcSwap;
    use crate::domain::models::{AppConfig, ButtonStats, LogicalKey, SwitchData, UserProfile};
    use crate::usecase::monitor::MonitorCommand;
    use crate::usecase::state_publisher::MonitorSharedState;
    use crate::usecase::test_helpers::{create_test_service, create_controllable_service};

    // --- Tests ---

    #[test]
    fn test_monitor_initialization() {
        let harness = create_test_service(UserProfile::default());
        // If we get here without panic, initialization succeeded
        let _ = harness.service;
    }

    #[test]
    fn test_conflict_resolution() {
        let mut profile = UserProfile::default();
        // Assign Key1 -> Button 1
        profile.controllers.get_mut("default").unwrap().mapping.bindings.insert(LogicalKey::Key1, 1);

        let harness = create_test_service(profile);
        let mut service = harness.service;

        // Attempt to assign Key2 -> Button 1 (Conflict)
        service.handle_command(MonitorCommand::SetKeyBinding { key: LogicalKey::Key2, button: 1 });

        // Key1 should be unbound (set to 0)
        let active = service.profile.controllers.get(&service.profile.active_controller_id).unwrap();
        assert_eq!(active.mapping.bindings.get(&LogicalKey::Key1), Some(&0));
        // Key2 should be inserted
        assert_eq!(active.mapping.bindings.get(&LogicalKey::Key2), Some(&1));
    }

    #[test]
    fn test_session_reset_integration() {
        // This test runs the actual monitor loop in a thread to verify session reset logic
        let mut profile = UserProfile::default();
        let key = LogicalKey::Key1;

        // Bind Key1 to Button 1
        profile.controllers.get_mut("default").unwrap().mapping.bindings.insert(key.clone(), 1);

        // Pre-populate stats with some session presses
        let mut stats = ButtonStats::default();
        stats.total_presses = 100;
        stats.last_session_presses = 50;
        profile.controllers.get_mut("default").unwrap().switches.insert(key.clone(), SwitchData {
            switch_model_id: "test".to_string(),
            stats,
            last_replaced_at: None,
        });
        // Set update frequency very high for test
        profile.config.polling_rate_ms_connected = 1;

        let harness = create_controllable_service(profile);
        let shared_state = harness.shared_state.clone();
        let tx = harness.tx.clone();
        let process_running = harness.process_running.clone();

        let service = harness.service;

        // Run monitor in background thread
        let handle = thread::spawn(move || {
            service.run();
        });

        // 1. Verify initial state (via shared state) with Retry Loop
        let mut found = false;
        for _ in 0..40 { // Wait up to 2s
            thread::sleep(Duration::from_millis(50));
            let snapshot = shared_state.load();
            if let Some(data) = snapshot.switches.get(&key) {
                if data.stats.last_session_presses == 50 {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Timed out waiting for initial state with 50 session presses");

        // 2. Start Game -> Should reset session stats
        *process_running.lock().unwrap() = true;

        // Wait for polling (process check interval is 2s by default in code)
        thread::sleep(Duration::from_millis(2100));

        let snapshot = shared_state.load();
        if let Some(data) = snapshot.switches.get(&key) {
             assert_eq!(data.stats.last_session_presses, 0, "Session presses should be reset after game start");
        } else {
             panic!("Key missing from stats after game start");
        }

        // 3. Shutdown
        tx.send(MonitorCommand::Shutdown).unwrap();
        handle.join().unwrap();
    }

    #[test]
    fn test_handle_command_update_config() {
        let harness = create_test_service(UserProfile::default());
        let mut service = harness.service;

        // Verify initial config
        assert_eq!(service.profile.config.target_controller_index, 0);

        // Send UpdateConfig command
        let mut new_config = AppConfig::default();
        new_config.target_controller_index = 2;
        new_config.target_process_name = "test_game.exe".to_string();

        service.handle_command(MonitorCommand::UpdateConfig(new_config.clone()));

        // Verify internal state updated
        assert_eq!(service.profile.config.target_controller_index, 2);
        assert_eq!(service.profile.config.target_process_name, "test_game.exe");
    }

    #[test]
    fn test_reset_stats_and_replace_switch() {
        use chrono::Utc;

        let mut profile = UserProfile::default();
        let key = LogicalKey::Key1;
        profile.controllers.get_mut("default").unwrap().switches.insert(key.clone(), SwitchData {
            switch_model_id: "old_model".to_string(),
            stats: ButtonStats {
                total_presses: 100,
                total_chatters: 10,
                ..Default::default()
            },
            last_replaced_at: None,
        });

        let harness = create_test_service(profile);
        let mut service = harness.service;

        // 1. Test ResetStats
        service.handle_command(MonitorCommand::ResetStats { key: key.clone() });

        let active_id = service.profile.active_controller_id.clone();
        let active = service.profile.controllers.get(&active_id).unwrap();
        let switch = active.switches.get(&key).unwrap();
        assert_eq!(switch.stats.total_presses, 0);
        assert_eq!(switch.stats.total_chatters, 0);
        assert_eq!(switch.switch_model_id, "old_model");
        assert!(switch.last_replaced_at.is_some());

        // History check
        assert_eq!(active.switch_history.len(), 1);
        assert_eq!(active.switch_history[0].event_type, "Reset");
        assert_eq!(active.switch_history[0].previous_stats.total_presses, 100);

        // Simulate usage again
        service.profile.controllers.get_mut(&active_id).unwrap().switches.get_mut(&key).unwrap().stats.total_presses = 50;

        // 2. Test ReplaceSwitch
        service.handle_command(MonitorCommand::ReplaceSwitch {
            key: key.clone(),
            new_model_id: "new_model".to_string()
        });

        let active = service.profile.controllers.get(&active_id).unwrap();
        let switch = active.switches.get(&key).unwrap();
        assert_eq!(switch.stats.total_presses, 0);
        assert_eq!(switch.switch_model_id, "new_model");

        // History check
        assert_eq!(active.switch_history.len(), 2);
        assert_eq!(active.switch_history[1].event_type, "Replace");
        assert_eq!(active.switch_history[1].previous_stats.total_presses, 50);

        // 3. Test SetLastReplacedDate
        let now = Utc::now();
        service.handle_command(MonitorCommand::SetLastReplacedDate {
            key: key.clone(),
            date: now,
        });
        let active = service.profile.controllers.get(&active_id).unwrap();
        let switch = active.switches.get(&key).unwrap();
        assert_eq!(switch.last_replaced_at, Some(now));

        // History check
        assert_eq!(active.switch_history.len(), 3);
        assert_eq!(active.switch_history[2].event_type, "ManualEdit");
    }

    #[test]
    fn test_shared_state_includes_config() {
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));
        let snapshot = shared_state.load();
        let _config: &AppConfig = &snapshot.config;
        assert_eq!(_config.target_controller_index, 0);
    }
}
