use log::{error, info};
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::domain::errors::InputError;
use crate::domain::interfaces::InputSource;
use crate::domain::models::LogicalKey;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::infrastructure::timer::HighResolutionTimer;

use super::MonitorService;

impl<I: InputSource, P: ProcessMonitor, R: ConfigRepository> MonitorService<I, P, R> {
    pub(super) fn check_game_running(&mut self) -> bool {
        self.process_monitor
            .is_process_running(&self.profile.config.target_process_name)
    }

    pub(super) fn handle_connection_state(
        &mut self,
        input_result: &Result<u32, InputError>,
        was_connected: &mut bool,
        force_publish: &mut bool,
    ) -> bool {
        let is_connected = match input_result {
            Ok(_) => true,
            Err(InputError::Disconnected) => false,
            Err(e) => {
                error!("Input Error: {}", e);
                false
            }
        };

        if is_connected != *was_connected {
            info!(
                "Connection state changed: {} -> {}",
                *was_connected, is_connected
            );
            *was_connected = is_connected;
            *force_publish = true;

            if is_connected {
                if self.high_res_timer.is_none() {
                    self.high_res_timer = Some(HighResolutionTimer::new());
                    info!("High resolution timer enabled");
                }
            } else {
                if self.high_res_timer.is_some() {
                    self.high_res_timer = None;
                    info!("High resolution timer disabled");
                }
            }
        }

        is_connected
    }

    pub(super) fn check_game_status(
        &mut self,
        last_process_check: &mut Instant,
        process_check_interval: Duration,
        was_game_running: bool,
    ) -> bool {
        if last_process_check.elapsed() >= process_check_interval {
            let running = self.check_game_running();
            *last_process_check = Instant::now();
            running
        } else {
            was_game_running
        }
    }

    pub(super) fn process_input(
        &mut self,
        input_result: Result<u32, InputError>,
        is_game_running: bool,
        current_pressed_keys: &mut HashSet<LogicalKey>,
    ) -> u32 {
        current_pressed_keys.clear();
        let mut current_raw_buttons = 0;

        if let Ok(w_buttons) = input_result {
            current_raw_buttons = w_buttons;
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let active_profile = self.profile.controllers.get_mut(&self.profile.active_controller_id).unwrap();

            for (key, &mask) in &active_profile.mapping.bindings {
                let is_pressed = (w_buttons & mask) != 0;
                if is_pressed {
                    current_pressed_keys.insert(key.clone());
                }

                let switch_data =
                    active_profile.switches.entry(key.clone()).or_insert_with(|| {
                        crate::domain::models::SwitchData {
                            switch_model_id: "generic_unknown".to_string(),
                            stats: crate::domain::models::ButtonStats::default(),
                            last_replaced_at: None,
                        }
                    });

                self.chatter_detector.process_button(
                    key,
                    is_pressed,
                    now_ms,
                    &mut switch_data.stats,
                    is_game_running,
                );
            }
        }
        current_raw_buttons
    }
}
