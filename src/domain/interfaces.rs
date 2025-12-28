use crate::domain::errors::InputError;
use crate::domain::models::InputMethod;

/// Abstraction for getting input state.
pub trait InputSource: Send {
    /// Returns a bitmask of pressed buttons.
    /// Returns Err(InputError::Disconnected) if device is disconnected.
    fn get_state(&mut self, controller_index: u32) -> Result<u16, InputError>;

    /// Optional: updates the input method if the source supports switching.
    fn set_input_method(&mut self, _method: InputMethod) {}
}
