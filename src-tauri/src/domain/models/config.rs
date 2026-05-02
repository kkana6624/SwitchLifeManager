use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMethod {
    XInput,
    DirectInput,
}

impl Default for InputMethod {
    fn default() -> Self {
        Self::DirectInput
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub target_controller_index: u32,
    pub input_method: InputMethod,
    pub chatter_threshold_ms: u64,
    pub polling_rate_ms_connected: u64,
    pub polling_rate_ms_disconnected: u64,
    pub target_process_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            target_controller_index: 0,
            input_method: InputMethod::default(),
            chatter_threshold_ms: 15,
            polling_rate_ms_connected: 1,
            polling_rate_ms_disconnected: 1000,
            target_process_name: "bm2dx.exe".to_string(),
        }
    }
}
