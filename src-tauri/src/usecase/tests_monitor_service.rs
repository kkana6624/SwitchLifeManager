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
        fn get_state(&mut self, _controller_index: u32) -> Result<u16, InputError> {
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

        // Note: handle_command does NOT automatically publish state. The loop does.
        // So shared_state is NOT updated yet. This is expected behavior of the architecture.
        // We can manually trigger publish_state if it were public, but it's private.
        // However, we can run the loop for a short time or verify via `service.profile` which we did.
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
