//! Input source implementations for different controller backends.
//!
//! - `GilrsInputSource` — DirectInput/HID via the `gilrs` crate (default)
//! - `XInputSource` — Windows XInput (Windows only)
//! - `MockInputSource` — Test mock (`#[cfg(test)]` only)
//! - `DynamicInputSource` — Runtime-switchable enum wrapping the above

mod gilrs_source;
#[cfg(target_os = "windows")]
mod xinput_source;
#[cfg(test)]
mod mock_source;

pub use gilrs_source::GilrsInputSource;
#[cfg(target_os = "windows")]
pub use xinput_source::XInputSource;
#[cfg(test)]
pub use mock_source::MockInputSource;

use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use crate::domain::models::InputMethod;

/// Runtime-switchable input source that delegates to the appropriate backend.
pub enum DynamicInputSource {
    #[cfg(target_os = "windows")]
    XInput(XInputSource),
    Gilrs(GilrsInputSource),
    #[cfg(test)]
    Mock(MockInputSource),
}

impl DynamicInputSource {
    pub fn new(method: InputMethod) -> Self {
        match method {
            #[cfg(target_os = "windows")]
            InputMethod::XInput => Self::XInput(XInputSource::new()),
            #[cfg(not(target_os = "windows"))]
            InputMethod::XInput => {
                // Fallback for non-windows
                 Self::Gilrs(GilrsInputSource::new())
            },
            InputMethod::DirectInput => Self::Gilrs(GilrsInputSource::new()),
        }
    }

    // Allow switching at runtime
    pub fn switch_to(&mut self, method: InputMethod) {
        *self = Self::new(method);
    }
}

impl InputSource for DynamicInputSource {
    fn get_state(&mut self, controller_index: u32) -> Result<u32, InputError> {
        match self {
            #[cfg(target_os = "windows")]
            Self::XInput(s) => s.get_state(controller_index),
            Self::Gilrs(s) => s.get_state(controller_index),
            #[cfg(test)]
            Self::Mock(s) => s.get_state(controller_index),
        }
    }

    fn set_input_method(&mut self, method: InputMethod) {
        self.switch_to(method);
    }

    fn enumerate_controllers(&mut self) -> Result<Vec<crate::domain::models::ControllerInfo>, InputError> {
        match self {
            #[cfg(target_os = "windows")]
            Self::XInput(_) => {
                // For XInput, we'll instantiate a temporary Gilrs to enumerate all connected gamepads
                // because XInput doesn't provide hardware UUIDs easily.
                let temp_gilrs = gilrs::Gilrs::new().map_err(|_| InputError::Disconnected)?;
                let mut controllers = Vec::new();
                for (_id, gamepad) in temp_gilrs.gamepads() {
                    let uuid_bytes = gamepad.uuid();
                    let uuid_str = uuid_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                    controllers.push(crate::domain::models::ControllerInfo {
                        id: uuid_str,
                        name: gamepad.name().to_string(),
                    });
                }
                Ok(controllers)
            },
            Self::Gilrs(s) => s.enumerate_controllers(),
            #[cfg(test)]
            Self::Mock(s) => s.enumerate_controllers(),
        }
    }
}
