use sysinfo::{ProcessesToUpdate, System};
use std::ffi::OsStr;

/// Abstraction for monitoring the target game process.
pub trait ProcessMonitor: Send + Sync {
    /// Checks if the target process is currently running.
    fn is_process_running(&mut self, process_name: &str) -> bool;
}

/// Implementation using sysinfo.
pub struct SysinfoProcessMonitor {
    system: System,
}

impl SysinfoProcessMonitor {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl ProcessMonitor for SysinfoProcessMonitor {
    fn is_process_running(&mut self, process_name: &str) -> bool {
        // Refresh only processes to reduce overhead.
        // In 0.30+, we specify what to update.
        // We only need the list of processes, no specific info like CPU/Memory is critical if we just check existence.
        // However, checking existence usually implies getting the list.
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        let process_name_os = OsStr::new(process_name);

        // Find process by name
        let mut found = false;
        for process in self.system.processes_by_name(process_name_os) {
            // If we are here, it matches the name (at least partially or exactly depending on OS).
            // sysinfo processes_by_name usually does exact match on Name.
            // We can just return true.
            let _ = process; // suppress unused
            found = true;
            break;
        }

        found
    }
}

/// Mock implementation for testing.
pub struct MockProcessMonitor {
    pub is_running: bool,
}

impl MockProcessMonitor {
    pub fn new(is_running: bool) -> Self {
        Self { is_running }
    }

    pub fn set_running(&mut self, running: bool) {
        self.is_running = running;
    }
}

impl ProcessMonitor for MockProcessMonitor {
    fn is_process_running(&mut self, _process_name: &str) -> bool {
        self.is_running
    }
}
