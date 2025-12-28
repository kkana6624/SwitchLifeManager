use crate::domain::errors::InputError;

/// Abstraction for getting input state.
pub trait InputSource: Send + Sync {
    /// Returns a bitmask of pressed buttons.
    /// Returns Err(InputError::Disconnected) if device is disconnected.
    fn get_state(&mut self, controller_index: u32) -> Result<u16, InputError>;
}
