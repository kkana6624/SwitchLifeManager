use log::{error, info};
use std::sync::Arc;
use std::time::Duration;

use crate::domain::interfaces::InputSource;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::usecase::input_monitor::ChatterDetector;
use crate::usecase::switch_operations::SwitchOperations;

use super::commands::MonitorCommand;
use super::MonitorService;

impl<I: InputSource, P: ProcessMonitor, R: ConfigRepository> MonitorService<I, P, R> {
    // Made public for testing
    pub fn handle_command(&mut self, cmd: MonitorCommand) {
        match cmd {
            MonitorCommand::Shutdown => {
                info!("Shutdown command received");
            }
            MonitorCommand::ForceSave => {
                if let Err(e) = self.repository.save(&self.profile) {
                    error!("Force save failed: {}", e);
                    let msg = format!("Save failed: {}", e);
                    self.publisher.update_status(msg.clone());
                    self.publisher.update_save_result(false, msg);
                } else {
                    let msg = "Saved successfully".to_string();
                    self.publisher.update_status(msg.clone());
                    self.publisher.update_save_result(true, msg);
                }
            }
            MonitorCommand::UpdateConfig(cfg) => {
                // Update input method if changed
                if cfg.input_method != self.profile.config.input_method {
                    self.input_source.set_input_method(cfg.input_method.clone());
                    info!("Input method switched to {:?}", cfg.input_method);
                }

                self.profile.config = cfg;
                self.chatter_detector =
                    ChatterDetector::new(self.profile.config.chatter_threshold_ms);
                info!("Config updated");
            }
            MonitorCommand::UpdateMapping(name, bindings) => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    active_profile.mapping.profile_name = name;
                    active_profile.mapping.bindings = bindings;
                    // Update cached bindings
                    self.cached_bindings = Arc::new(active_profile.mapping.bindings.clone());
                    info!("Mapping updated");
                }
            }
            MonitorCommand::SetKeyBinding { key, button } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    let mut conflict_key = None;
                    for (k, &v) in &active_profile.mapping.bindings {
                        if v == button && *k != key {
                            conflict_key = Some(k.clone());
                            break;
                        }
                    }

                    if let Some(old_key) = conflict_key {
                        active_profile.mapping.bindings.insert(old_key.clone(), 0);
                        info!(
                            "Unbound duplicate binding for key: {} (was button {})",
                            old_key, button
                        );
                    }

                    active_profile.mapping.bindings.insert(key.clone(), button);
                    self.cached_bindings = Arc::new(active_profile.mapping.bindings.clone());
                    info!("Set binding for key: {} -> button {}", key, button);
                }
            }
            MonitorCommand::ReplaceSwitch { key, new_model_id } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    SwitchOperations::replace_switch(active_profile, key.clone(), new_model_id.clone());
                    info!("Replaced switch for {} with new model {}", key, new_model_id);
                }
            }
            MonitorCommand::ResetStats { key } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    SwitchOperations::reset_stats(active_profile, key.clone());
                    info!("Reset stats for {}", key);
                }
            }
            MonitorCommand::SetLastReplacedDate { key, date } => {
                if let Some(active_profile) = self.profile.controllers.get_mut(&self.profile.active_controller_id) {
                    SwitchOperations::set_last_replaced_date(active_profile, key.clone(), date);
                    info!("Set last replaced date for {} to {}", key, date);
                }
            }
            MonitorCommand::SetActiveController(id) => {
                info!("Setting active controller to: {}", id);
                self.profile.active_controller_id = id.clone();
                // Ensure profile exists
                if !self.profile.controllers.contains_key(&id) {
                    self.profile.controllers.insert(id.clone(), crate::domain::models::ControllerProfile::default());
                }
                let active_profile = self.profile.controllers.get(&id).unwrap();
                self.cached_bindings = Arc::new(active_profile.mapping.bindings.clone());
            }
        }
    }

    pub(super) fn process_commands(&mut self, force_publish: &mut bool) -> bool {
        while let Ok(cmd) = self.command_rx.try_recv() {
            if let MonitorCommand::Shutdown = cmd {
                info!("Shutdown command received");
                return false;
            }
            self.handle_command(cmd);
            *force_publish = true;
        }
        true
    }

    pub(super) fn wait_for_next_poll(&mut self, was_connected: bool, force_publish: &mut bool) -> bool {
        let polling_rate = if was_connected {
            self.profile.config.polling_rate_ms_connected
        } else {
            self.profile.config.polling_rate_ms_disconnected
        };

        use crossbeam_channel::select;
        select! {
            recv(self.command_rx) -> msg => {
                 if let Ok(cmd) = msg {
                     if let MonitorCommand::Shutdown = cmd {
                         info!("Shutdown command received during wait");
                         return false;
                     } else {
                         self.handle_command(cmd);
                         *force_publish = true;
                     }
                 }
            },
            default(Duration::from_millis(polling_rate)) => {
                // Timeout elapsed
            }
        }
        true
    }
}
