use anyhow::Result;

/// Abstraction for getting input state.
/// This allows us to mock the XInput interactions for testing on Linux.
pub trait InputSource: Send + Sync {
    /// Returns a bitmask of pressed buttons (similar to XInput wButtons).
    /// Returns Err if device is disconnected.
    fn get_state(&mut self, controller_index: u32) -> Result<u16>;
}

pub struct MockInputSource {
    pub states: Vec<Result<u16>>,
    pub current_index: usize,
}

impl MockInputSource {
    pub fn new(states: Vec<Result<u16>>) -> Self {
        Self {
            states,
            current_index: 0,
        }
    }
}

impl InputSource for MockInputSource {
    fn get_state(&mut self, _controller_index: u32) -> Result<u16> {
        if self.current_index < self.states.len() {
            let result = &self.states[self.current_index];
            self.current_index += 1;
            match result {
                Ok(v) => Ok(*v),
                Err(e) => Err(anyhow::anyhow!("Mock Error: {}", e)),
            }
        } else {
            // Default to 0 or Error? Let's default to 0 (no input)
            Ok(0)
        }
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
    fn get_state(&mut self, controller_index: u32) -> Result<u16> {
        use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};

        let mut state = XINPUT_STATE::default();
        // XInputGetState returns distinct error codes for connection.
        // ERROR_SUCCESS (0) or ERROR_DEVICE_NOT_CONNECTED (1167)
        let result = unsafe { XInputGetState(controller_index, &mut state) };

        if result == 0 { // ERROR_SUCCESS
            // wButtons is usually u16 but might be wrapped in XINPUT_GAMEPAD_BUTTON_FLAGS
            // we need to cast it to u16 or extract the value.
            Ok(state.Gamepad.wButtons.0 as u16)
        } else if result == 1167 { // ERROR_DEVICE_NOT_CONNECTED
             Err(anyhow::anyhow!("Device not connected"))
        } else {
             Err(anyhow::anyhow!("XInput Error: {}", result))
        }
    }
}
