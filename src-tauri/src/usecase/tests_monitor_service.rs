#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crossbeam_channel::bounded;
    use arc_swap::ArcSwap;
    use anyhow::Result;
    use crate::domain::models::{AppConfig, UserProfile};
    use crate::domain::interfaces::InputSource;
    use crate::domain::errors::InputError;
    use crate::infrastructure::persistence::ConfigRepository;
    use crate::infrastructure::process_monitor::ProcessMonitor;
    use crate::usecase::monitor::{MonitorService, MonitorCommand, MonitorSharedState};

    // --- Mocks ---

    #[derive(Clone)]
    struct MockInputSource;

    impl InputSource for MockInputSource {
        fn get_state(&mut self, _controller_index: u32) -> Result<u32, InputError> {
            Ok(0)
        }
    }

    #[derive(Clone)]
    struct MockProcessMonitor;

    impl ProcessMonitor for MockProcessMonitor {
        fn is_process_running(&mut self, _process_name: &str) -> bool {
            false
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
    fn test_handle_command_update_config() {
        let (_tx, rx) = bounded(10);
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

        let repo = MockRepository {
            profile: UserProfile::default(),
        };
        let input = MockInputSource;
        let process = MockProcessMonitor;

        let mut service = MonitorService::new(input, process, repo, rx, shared_state.clone()).unwrap();

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
        use crate::domain::models::{LogicalKey, SwitchData, ButtonStats};
        use std::collections::HashMap;
        use chrono::{DateTime, Utc};

        let (_tx, rx) = bounded(10);
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));

        // Setup profile with some data
        let mut profile = UserProfile::default();
        let key = LogicalKey::Key1;
        profile.switches.insert(key.clone(), SwitchData {
            switch_model_id: "old_model".to_string(),
            stats: ButtonStats {
                total_presses: 100,
                total_chatters: 10,
                ..Default::default()
            },
            last_replaced_at: None,
        });

        let repo = MockRepository { profile };
        let input = MockInputSource;
        let process = MockProcessMonitor;

        let mut service = MonitorService::new(input, process, repo, rx, shared_state.clone()).unwrap();

        // 1. Test ResetStats
        service.handle_command(MonitorCommand::ResetStats { key: key.clone() });
        
        let switch = service.profile.switches.get(&key).unwrap();
        assert_eq!(switch.stats.total_presses, 0);
        assert_eq!(switch.stats.total_chatters, 0);
        assert_eq!(switch.switch_model_id, "old_model");
        assert!(switch.last_replaced_at.is_some());
        
        // History check
        assert_eq!(service.profile.switch_history.len(), 1);
        assert_eq!(service.profile.switch_history[0].event_type, "Reset");
        assert_eq!(service.profile.switch_history[0].previous_stats.total_presses, 100);

        // Simulate usage again
        service.profile.switches.get_mut(&key).unwrap().stats.total_presses = 50;

        // 2. Test ReplaceSwitch
        service.handle_command(MonitorCommand::ReplaceSwitch { 
            key: key.clone(), 
            new_model_id: "new_model".to_string() 
        });

        let switch = service.profile.switches.get(&key).unwrap();
        assert_eq!(switch.stats.total_presses, 0);
        assert_eq!(switch.switch_model_id, "new_model");
        
        // History check
        assert_eq!(service.profile.switch_history.len(), 2);
        assert_eq!(service.profile.switch_history[1].event_type, "Replace");
        assert_eq!(service.profile.switch_history[1].previous_stats.total_presses, 50);

        // 3. Test SetLastReplacedDate
        let now = Utc::now();
        service.handle_command(MonitorCommand::SetLastReplacedDate {
            key: key.clone(),
            date: now,
        });
        let switch = service.profile.switches.get(&key).unwrap();
        assert_eq!(switch.last_replaced_at, Some(now));
        
        // History check
        assert_eq!(service.profile.switch_history.len(), 3);
        assert_eq!(service.profile.switch_history[2].event_type, "ManualEdit");
    }

    #[test]
    fn test_shared_state_includes_config() {
        // This validates the glue code change: MonitorSharedState must have 'config' field.
        
        let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));
        // Just compiling this accesses the field, proving existence.
        let snapshot = shared_state.load();
        let _config: &AppConfig = &snapshot.config; 
        
        // Let's verify default values match
        assert_eq!(_config.target_controller_index, 0);
    }
}
