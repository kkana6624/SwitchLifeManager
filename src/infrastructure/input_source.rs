use anyhow::Result;
use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use std::collections::VecDeque;

pub struct MockInputSource {
    pub states: VecDeque<Result<u16, InputError>>,
}

impl MockInputSource {
    pub fn new(states: Vec<Result<u16, InputError>>) -> Self {
        Self {
            states: states.into(),
        }
    }
}

impl InputSource for MockInputSource {
    fn get_state(&mut self, _controller_index: u32) -> Result<u16, InputError> {
        self.states.pop_front().unwrap_or(Ok(0))
    }
}

// Windows Implementation
#[cfg(target_os = "windows")]
pub struct XInputSource;

#[cfg(target_os = "windows")]
impl XInputSource {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
impl InputSource for XInputSource {
    fn get_state(&mut self, controller_index: u32) -> Result<u16, InputError> {
        use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};

        let mut state = XINPUT_STATE::default();
        let result = unsafe { XInputGetState(controller_index, &mut state) };

        if result == 0 { // ERROR_SUCCESS
            Ok(state.Gamepad.wButtons.0 as u16)
        } else if result == 1167 { // ERROR_DEVICE_NOT_CONNECTED
             Err(InputError::Disconnected)
        } else {
             Err(InputError::Other(anyhow::anyhow!("XInput Error: {}", result)))
        }
    }
}
