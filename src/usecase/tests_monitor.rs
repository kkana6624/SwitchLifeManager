
#[cfg(test)]
pub mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use std::thread;
    use crossbeam_channel::bounded;
    use arc_swap::ArcSwap;
    use anyhow::Result;
    use crate::domain::models::{ButtonStats, LogicalKey, UserProfile};
    use crate::domain::interfaces::InputSource;
    use crate::domain::errors::InputError;
    use crate::infrastructure::persistence::ConfigRepository;
    use crate::infrastructure::process_monitor::ProcessMonitor;
    use crate::usecase::monitor::{MonitorService, MonitorCommand, MonitorSharedState};

    // --- Mocks ---

    #[derive(Clone)]
    struct MockInputSource {
        pub state_val: Arc<Mutex<u16>>,
        pub is_disconnected: Arc<Mutex<bool>>,
    }

    impl InputSource for MockInputSource {
        fn get_state(&mut self, _controller_index: u32) -> Result<u16, InputError> {
            let is_disc = *self.is_disconnected.lock().unwrap();
            if is_disc {
                Err(InputError::Disconnected)
            } else {
                let val = *self.state_val.lock().unwrap();
                Ok(val)
            }
        }
    }

    #[derive(Clone)]
    struct MockProcessMonitor {
        pub is_running: Arc<Mutex<bool>>,
    }

    impl ProcessMonitor for MockProcessMonitor {
        fn is_process_running(&mut self, _process_name: &str) -> bool {
            *self.is_running.lock().unwrap()
        }
    }

    struct MockRepository {
        pub profile: UserProfile,
    }

    impl ConfigRepository for MockRepository {
        fn load(&self) -> Result<UserProfile> {
            Ok(self.profile.clone())
        }
        fn save(&self, _profile: &UserProfile) -> Result<()> {
            Ok(())
        }
    }

    // --- Tests ---

    #[test]
    fn test_monitor_initialization() {
        let (_tx, rx) = bounded(10);
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

        let repo = MockRepository {
            profile: UserProfile::default(),
        };
        let input = MockInputSource {
            state_val: Arc::new(Mutex::new(0)),
            is_disconnected: Arc::new(Mutex::new(false))
        };
        let process = MockProcessMonitor {
            is_running: Arc::new(Mutex::new(false))
        };

        let service = MonitorService::new(input, process, repo, rx, shared_state.clone());
        assert!(service.is_ok());
    }

    #[test]
    fn test_handle_command_audit_and_reset() {
        let (_tx, rx) = bounded(10);
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

        let mut profile = UserProfile::default();
        let key = LogicalKey::Key1;

        // Setup initial switch state
        let mut stats = ButtonStats::default();
        stats.total_presses = 100;
        stats.total_chatters = 5;

        profile.switches.insert(key.clone(), crate::domain::models::SwitchData {
            switch_model_id: "old_model".to_string(),
            stats: stats.clone(),
        });

        let repo = MockRepository { profile: profile.clone() };
        let input = MockInputSource {
            state_val: Arc::new(Mutex::new(0)),
            is_disconnected: Arc::new(Mutex::new(false))
        };
        let process = MockProcessMonitor {
            is_running: Arc::new(Mutex::new(false))
        };

        let mut service = MonitorService::new(input, process, repo, rx, shared_state.clone()).unwrap();

        // 1. Test ResetStats
        service.handle_command(MonitorCommand::ResetStats { key: key.clone() });

        // Check internal profile state
        let switch = service.profile.switches.get(&key).unwrap();
        assert_eq!(switch.stats.total_presses, 0);
        assert_eq!(switch.stats.total_chatters, 0);
        assert_eq!(switch.switch_model_id, "old_model"); // Model ID should persist

        // 2. Test ReplaceSwitch
        // First simulate some usage again
        service.profile.switches.get_mut(&key).unwrap().stats.total_presses = 50;

        service.handle_command(MonitorCommand::ReplaceSwitch {
            key: key.clone(),
            new_model_id: "new_model".to_string()
        });

        let switch = service.profile.switches.get(&key).unwrap();
        assert_eq!(switch.stats.total_presses, 0); // Should be reset
        assert_eq!(switch.switch_model_id, "new_model");
    }

    #[test]
    fn test_conflict_resolution() {
        let (_tx, rx) = bounded(10);
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

        let mut profile = UserProfile::default();
        // Assign Key1 -> Button 1
        profile.mapping.bindings.insert(LogicalKey::Key1, 1);

        let repo = MockRepository { profile };
        let input = MockInputSource {
            state_val: Arc::new(Mutex::new(0)),
            is_disconnected: Arc::new(Mutex::new(false))
        };
        let process = MockProcessMonitor {
            is_running: Arc::new(Mutex::new(false))
        };

        let mut service = MonitorService::new(input, process, repo, rx, shared_state.clone()).unwrap();

        // Attempt to assign Key2 -> Button 1 (Conflict)
        service.handle_command(MonitorCommand::SetKeyBinding { key: LogicalKey::Key2, button: 1 });

        // Key1 should be removed
        assert!(!service.profile.mapping.bindings.contains_key(&LogicalKey::Key1));
        // Key2 should be inserted
        assert_eq!(service.profile.mapping.bindings.get(&LogicalKey::Key2), Some(&1));
    }

    #[test]
    fn test_session_reset_integration() {
        // This test runs the actual monitor loop in a thread to verify session reset logic
        let (tx, rx) = bounded(10);
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

        let mut profile = UserProfile::default();
        let key = LogicalKey::Key1;
        // Bind Key1 to Button 1
        profile.mapping.bindings.insert(key.clone(), 1);

        // Pre-populate stats with some session presses
        let mut stats = ButtonStats::default();
        stats.total_presses = 100;
        stats.last_session_presses = 50;
        profile.switches.insert(key.clone(), crate::domain::models::SwitchData {
            switch_model_id: "test".to_string(),
            stats: stats,
        });
        // Set update frequency very high for test
        profile.config.polling_rate_ms_connected = 1;

        let repo = MockRepository { profile };

        let state_val = Arc::new(Mutex::new(0));
        let is_running = Arc::new(Mutex::new(false)); // Initially NOT running

        let input = MockInputSource {
            state_val: state_val.clone(),
            is_disconnected: Arc::new(Mutex::new(false))
        };
        let process = MockProcessMonitor {
            is_running: is_running.clone()
        };

        let service = MonitorService::new(input, process, repo, rx, shared_state.clone()).unwrap();

        // Run monitor in background thread
        let handle = thread::spawn(move || {
            service.run();
        });

        // 1. Verify initial state (via shared state) with Retry Loop
        let mut found = false;
        for _ in 0..20 { // Wait up to 1s (20 * 50ms)
            thread::sleep(Duration::from_millis(50));
            let snapshot = shared_state.load();
            if let Some(stats) = snapshot.switch_stats.get(&key) {
                if stats.last_session_presses == 50 {
                    found = true;
                    break;
                }
            }
        }
        assert!(found, "Timed out waiting for initial state with 50 session presses");

        // 2. Start Game -> Should reset session stats
        *is_running.lock().unwrap() = true;

        // Wait for polling (process check interval is 2s by default in code)
        // We wait slightly longer than 2s to ensure the process check triggers
        thread::sleep(Duration::from_millis(2100));

        let snapshot = shared_state.load();
        if let Some(stats) = snapshot.switch_stats.get(&key) {
             assert_eq!(stats.last_session_presses, 0, "Session presses should be reset after game start");
        } else {
             panic!("Key missing from stats after game start");
        }

        // 3. Stop Game -> Logic (logs)
        // Shutdown
        tx.send(MonitorCommand::Shutdown).unwrap();
        handle.join().unwrap();
    }
}
