use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;

/// Windows XInput input source for Xbox-compatible controllers.
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
    fn get_state(&mut self, controller_index: u32) -> Result<u32, InputError> {
        use windows::Win32::UI::Input::XboxController::{XInputGetState, XINPUT_STATE};

        let mut state = XINPUT_STATE::default();
        let result = unsafe { XInputGetState(controller_index, &mut state) };

        if result == 0 { // ERROR_SUCCESS
            Ok(state.Gamepad.wButtons.0 as u32)
        } else if result == 1167 { // ERROR_DEVICE_NOT_CONNECTED
             Err(InputError::Disconnected)
        } else {
             Err(InputError::Other(anyhow::anyhow!("XInput Error: {}", result)))
        }
    }
}
