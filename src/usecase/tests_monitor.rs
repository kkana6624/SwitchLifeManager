use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use crossbeam_channel::unbounded;
use crate::domain::models::{AppConfig, LogicalKey, UserProfile};
use crate::infrastructure::input_source::MockInputSource;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::MockProcessMonitor;
use crate::usecase::monitor::{MonitorCommand, MonitorService, MonitorSharedState};
use crate::domain::errors::InputError;
use anyhow::Result;

// Mock Repository
struct MockRepository {
    profile: Arc<RwLock<UserProfile>>,
}

impl MockRepository {
    fn new() -> Self {
        Self {
            profile: Arc::new(RwLock::new(UserProfile::default())),
        }
    }
}

impl ConfigRepository for MockRepository {
    fn load(&self) -> Result<UserProfile> {
        Ok(self.profile.read().unwrap().clone())
    }

    fn save(&self, profile: &UserProfile) -> Result<()> {
        *self.profile.write().unwrap() = profile.clone();
        Ok(())
    }
}

#[test]
fn test_monitor_loop_integration() {
    // 1. Setup Dependencies
    // Input sequence: Disconnected -> Connected (No Input) -> Connected (Input) -> Disconnected
    // Note: MockInputSource returns Ok(val) or Err.
    // XInput returns 0 (connected) or error code.
    // Our MockInputSource wraps this: Ok(val) = connected with input. Err = disconnected.

    let inputs = vec![
        Err(InputError::Disconnected), // 1
        Ok(0), // 2 Connected, no press
        Ok(0), // 3
        Ok(1), // 4 Connected, press button 1 (bitmask 1)
        Ok(1), // 5 Hold
        Ok(0), // 6 Release
        Err(InputError::Disconnected), // 7
    ];

    let input_source = MockInputSource::new(inputs);
    let process_monitor = MockProcessMonitor::new(false);
    let repo = MockRepository::new();

    // Setup Profile with mapping for bitmask 1 -> Key1
    {
        let mut p = repo.profile.write().unwrap();
        p.mapping.bindings.insert(LogicalKey::Key1, 1);
        p.config.polling_rate_ms_connected = 1; // Fast for test
        p.config.polling_rate_ms_disconnected = 1; // Fast for test
    }

    let (tx, rx) = unbounded();
    let shared_state = Arc::new(RwLock::new(MonitorSharedState::default()));

    let service = MonitorService::new(
        input_source,
        process_monitor,
        repo,
        rx,
        shared_state.clone(),
    ).unwrap();

    // 2. Run Monitor in background thread
    let handle = thread::spawn(move || {
        service.run();
    });

    // 3. Wait a bit for processing
    thread::sleep(Duration::from_millis(100));

    // 4. Send Shutdown
    tx.send(MonitorCommand::Shutdown).unwrap();

    handle.join().unwrap();

    // 5. Verify State
    let state = shared_state.read().unwrap();

    let stats = state.switch_stats.get(&LogicalKey::Key1);
    assert!(stats.is_some(), "Key1 stats should exist");
    let stats = stats.unwrap();

    assert_eq!(stats.total_presses, 1, "Should have 1 press");
    assert_eq!(stats.total_releases, 1, "Should have 1 release");
}

#[test]
fn test_command_handling() {
    let input_source = MockInputSource::new(vec![]);
    let process_monitor = MockProcessMonitor::new(false);
    let repo = MockRepository::new();

    // Ensure polling rate is fast so sleep doesn't block command processing for too long
    {
        let mut p = repo.profile.write().unwrap();
        p.config.polling_rate_ms_connected = 1;
        p.config.polling_rate_ms_disconnected = 1;
    }

    let (tx, rx) = unbounded();
    let shared_state = Arc::new(RwLock::new(MonitorSharedState::default()));

    let service = MonitorService::new(
        input_source,
        process_monitor,
        repo,
        rx,
        shared_state.clone(),
    ).unwrap();

    let handle = thread::spawn(move || {
        service.run();
    });

    // Update Config
    let mut new_config = AppConfig::default();
    new_config.target_controller_index = 2;
    // Set polling rates fast here too so it doesn't slow down after update
    new_config.polling_rate_ms_connected = 1;
    new_config.polling_rate_ms_disconnected = 1;

    tx.send(MonitorCommand::UpdateConfig(new_config)).unwrap();

    thread::sleep(Duration::from_millis(100));

    {
        let state = shared_state.read().unwrap();
        assert_eq!(state.target_controller_index, 2);
    }

    tx.send(MonitorCommand::Shutdown).unwrap();
    handle.join().unwrap();
}

#[test]
fn test_set_key_binding_duplicate_resolution() {
    let input_source = MockInputSource::new(vec![]);
    let process_monitor = MockProcessMonitor::new(false);
    let repo = MockRepository::new();

    // Initial state: Key1 -> 1
    {
        let mut p = repo.profile.write().unwrap();
        p.mapping.bindings.insert(LogicalKey::Key1, 1);
        p.config.polling_rate_ms_connected = 1;
        p.config.polling_rate_ms_disconnected = 1;
    }

    let (tx, rx) = unbounded();
    let shared_state = Arc::new(RwLock::new(MonitorSharedState::default()));

    let service = MonitorService::new(
        input_source,
        process_monitor,
        repo,
        rx,
        shared_state.clone(),
    ).unwrap();

    let handle = thread::spawn(move || {
        service.run();
    });

    // Case 1: Assign Key2 -> 2 (Fresh)
    tx.send(MonitorCommand::SetKeyBinding { key: LogicalKey::Key2, button: 2 }).unwrap();
    thread::sleep(Duration::from_millis(20));

    {
        let state = shared_state.read().unwrap();
        assert_eq!(state.bindings.get(&LogicalKey::Key1), Some(&1));
        assert_eq!(state.bindings.get(&LogicalKey::Key2), Some(&2));
    }

    // Case 2: Assign Key3 -> 1 (Duplicate of Key1)
    // Expect: Key1 removed, Key3 set to 1
    tx.send(MonitorCommand::SetKeyBinding { key: LogicalKey::Key3, button: 1 }).unwrap();
    thread::sleep(Duration::from_millis(20));

    {
        let state = shared_state.read().unwrap();
        assert_eq!(state.bindings.get(&LogicalKey::Key1), None, "Key1 should be removed");
        assert_eq!(state.bindings.get(&LogicalKey::Key3), Some(&1), "Key3 should be set");
        assert_eq!(state.bindings.get(&LogicalKey::Key2), Some(&2), "Key2 should be untouched");
    }

    // Case 3: Move Key3 -> 3 (Update existing)
    tx.send(MonitorCommand::SetKeyBinding { key: LogicalKey::Key3, button: 3 }).unwrap();
    thread::sleep(Duration::from_millis(20));

    {
        let state = shared_state.read().unwrap();
        assert_eq!(state.bindings.get(&LogicalKey::Key3), Some(&3));
        // Previous binding (1) is now free
    }

    tx.send(MonitorCommand::Shutdown).unwrap();
    handle.join().unwrap();
}
