#[cfg(target_os = "windows")]
use windows::Win32::Media::{timeBeginPeriod, timeEndPeriod};

pub struct HighResolutionTimer {
    #[cfg(target_os = "windows")]
    active: bool,
}

impl HighResolutionTimer {
    pub fn new() -> Self {
        #[cfg(target_os = "windows")]
        unsafe {
            timeBeginPeriod(1);
        }

        Self {
            #[cfg(target_os = "windows")]
            active: true,
        }
    }
}

impl Drop for HighResolutionTimer {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        if self.active {
            unsafe {
                timeEndPeriod(1);
            }
        }
    }
}
