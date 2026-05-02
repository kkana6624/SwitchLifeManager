//! Shared test utilities for MonitorService tests.
//! Provides common mock implementations and factory helpers to reduce boilerplate.

use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Sender, Receiver};
use arc_swap::ArcSwap;
use anyhow::Result;

use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use crate::domain::models::UserProfile;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::usecase::monitor::{MonitorCommand, MonitorService};
use crate::usecase::state_publisher::{MonitorSharedState, StatePublisher};

// =============================================================================
// Mock Implementations
// =============================================================================

/// Configurable mock input source.
/// - `state_val`: the raw button bitmap to return from `get_state()`
/// - `is_disconnected`: if true, returns `InputError::Disconnected`
#[derive(Clone)]
pub struct MockInputSource {
    pub state_val: Arc<Mutex<u32>>,
    pub is_disconnected: Arc<Mutex<bool>>,
}

impl MockInputSource {
    /// Create a simple mock that always returns 0 (no buttons pressed).
    pub fn idle() -> Self {
        Self {
            state_val: Arc::new(Mutex::new(0)),
            is_disconnected: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a controllable mock with shared state handles.
    pub fn controllable() -> (Self, Arc<Mutex<u32>>, Arc<Mutex<bool>>) {
        let state_val = Arc::new(Mutex::new(0));
        let is_disconnected = Arc::new(Mutex::new(false));
        let source = Self {
            state_val: state_val.clone(),
            is_disconnected: is_disconnected.clone(),
        };
        (source, state_val, is_disconnected)
    }
}

impl InputSource for MockInputSource {
    fn get_state(&mut self, _controller_index: u32) -> Result<u32, InputError> {
        let is_disc = *self.is_disconnected.lock().unwrap();
        if is_disc {
            Err(InputError::Disconnected)
        } else {
            let val = *self.state_val.lock().unwrap();
            Ok(val)
        }
    }
}

/// Mock process monitor with controllable `is_running` state.
#[derive(Clone)]
pub struct MockProcessMonitor {
    pub is_running: Arc<Mutex<bool>>,
}

impl MockProcessMonitor {
    /// Create a mock that always reports game as not running.
    pub fn not_running() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Create a controllable mock with a shared state handle.
    pub fn controllable() -> (Self, Arc<Mutex<bool>>) {
        let is_running = Arc::new(Mutex::new(false));
        let monitor = Self {
            is_running: is_running.clone(),
        };
        (monitor, is_running)
    }
}

impl ProcessMonitor for MockProcessMonitor {
    fn is_process_running(&mut self, _process_name: &str) -> bool {
        *self.is_running.lock().unwrap()
    }
}

/// Mock repository that loads/saves from an in-memory profile.
pub struct MockRepository {
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

// =============================================================================
// Factory Helpers
// =============================================================================

/// All components needed for a test: the service, command sender, and shared state.
pub struct TestHarness {
    pub service: MonitorService<MockInputSource, MockProcessMonitor, MockRepository>,
    pub tx: Sender<MonitorCommand>,
    pub shared_state: Arc<ArcSwap<MonitorSharedState>>,
}

/// Create a test MonitorService with default profile and idle mocks.
/// Returns the service, command sender, and shared state for assertions.
pub fn create_test_service(profile: UserProfile) -> TestHarness {
    let (tx, rx) = bounded(10);
    let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));
    let publisher = StatePublisher::new(shared_state.clone());

    let service = MonitorService::new(
        MockInputSource::idle(),
        MockProcessMonitor::not_running(),
        MockRepository { profile },
        rx,
        publisher,
    )
    .expect("Failed to create test MonitorService");

    TestHarness {
        service,
        tx,
        shared_state,
    }
}

/// Create a test MonitorService with controllable input/process mocks.
/// Returns the harness plus the control handles for input state and process running.
pub struct ControllableTestHarness {
    pub service: MonitorService<MockInputSource, MockProcessMonitor, MockRepository>,
    pub tx: Sender<MonitorCommand>,
    pub rx: Receiver<MonitorCommand>,
    pub shared_state: Arc<ArcSwap<MonitorSharedState>>,
    pub input_state: Arc<Mutex<u32>>,
    pub input_disconnected: Arc<Mutex<bool>>,
    pub process_running: Arc<Mutex<bool>>,
}

pub fn create_controllable_service(profile: UserProfile) -> ControllableTestHarness {
    let (tx, rx) = bounded(10);
    let shared_state = Arc::new(ArcSwap::from_pointee(MonitorSharedState::default()));
    let publisher = StatePublisher::new(shared_state.clone());

    let (input, input_state, input_disconnected) = MockInputSource::controllable();
    let (process, process_running) = MockProcessMonitor::controllable();

    let service = MonitorService::new(
        input,
        process,
        MockRepository { profile },
        rx.clone(),
        publisher,
    )
    .expect("Failed to create test MonitorService");

    ControllableTestHarness {
        service,
        tx,
        rx,
        shared_state,
        input_state,
        input_disconnected,
        process_running,
    }
}
