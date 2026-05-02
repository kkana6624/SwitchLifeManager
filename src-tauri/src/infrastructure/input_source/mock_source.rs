use anyhow::Result;
use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use std::collections::VecDeque;

/// Mock input source for testing.
#[cfg(test)]
pub struct MockInputSource {
    pub states: VecDeque<Result<u32, InputError>>,
}

#[cfg(test)]
impl MockInputSource {
    pub fn new(states: Vec<Result<u32, InputError>>) -> Self {
        Self {
            states: states.into(),
        }
    }
}

#[cfg(test)]
impl InputSource for MockInputSource {
    fn get_state(&mut self, _controller_index: u32) -> Result<u32, InputError> {
        self.states.pop_front().unwrap_or(Ok(0))
    }

    fn enumerate_controllers(&mut self) -> Result<Vec<crate::domain::models::ControllerInfo>, InputError> {
        Ok(vec![crate::domain::models::ControllerInfo {
            id: "mock_uuid".to_string(),
            name: "Mock Controller".to_string(),
        }])
    }
}
