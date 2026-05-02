use log::{error, info};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::domain::interfaces::InputSource;
use crate::domain::models::LogicalKey;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;

use super::MonitorService;

impl<I: InputSource, P: ProcessMonitor, R: ConfigRepository> MonitorService<I, P, R> {
    pub(super) fn publish_current_state(
        &self,
        is_connected: bool,
        is_game_running: bool,
        pressed_keys: &HashSet<LogicalKey>,
        raw_buttons: u32,
        connected_controllers: &[crate::domain::models::ControllerInfo],
    ) {
        let active_profile = self.profile.controllers.get(&self.profile.active_controller_id).unwrap();

        self.publisher.publish(
            is_connected,
            is_game_running,
            self.profile.config.clone(),
            active_profile.mapping.profile_name.clone(),
            self.cached_bindings.clone(),
            active_profile.switches.clone(),
            Arc::new(active_profile.switch_history.clone()),
            pressed_keys.clone(),
            raw_buttons,
            active_profile.recent_sessions.clone(),
            self.profile.active_controller_id.clone(),
            connected_controllers.to_vec(),
        );
    }

    pub(super) fn handle_autosave(&mut self, last_save_at: &mut Instant, save_interval: Duration) {
        if last_save_at.elapsed() >= save_interval {
            if let Err(e) = self.repository.save(&self.profile) {
                error!("Auto save failed: {}", e);
                self.publisher.update_save_result(false, format!("Auto save failed: {}", e));
            } else {
                self.publisher.update_save_result(true, "Auto save succeeded".to_string());
            }
            *last_save_at = Instant::now();
        }
    }

    pub(super) fn handle_shutdown(&mut self) {
        info!("Monitor loop exiting. Saving profile...");
        if let Err(e) = self.repository.save(&self.profile) {
            error!("Exit save failed: {}", e);
            self.publisher.update_save_result(false, format!("Exit save failed: {}", e));
        } else {
            self.publisher.update_save_result(true, "Exit save succeeded".to_string());
        }
        self.high_res_timer = None;
    }
}
