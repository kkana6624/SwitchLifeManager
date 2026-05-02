use chrono::Utc;
use log::info;

use crate::domain::interfaces::InputSource;
use crate::infrastructure::persistence::ConfigRepository;
use crate::infrastructure::process_monitor::ProcessMonitor;
use crate::usecase::session_manager::SessionManager;

use super::MonitorService;

impl<I: InputSource, P: ProcessMonitor, R: ConfigRepository> MonitorService<I, P, R> {
    pub(super) fn handle_game_session(
        &mut self,
        is_game_running: bool,
        was_game_running: &mut bool,
        force_publish: &mut bool,
    ) {
        if is_game_running != *was_game_running {
            info!(
                "Game running state changed: {} -> {}",
                *was_game_running, is_game_running
            );

            let active_profile = self.profile.controllers.get_mut(&self.profile.active_controller_id).unwrap();

            if is_game_running {
                info!("Game started. Resetting session stats.");
                self.current_session_start = Some(Utc::now());
                SessionManager::start_session(active_profile);
            } else {
                let end_time = Utc::now();
                info!("Game ended.");

                if let Some(start_time) = self.current_session_start.take() {
                    let duration_secs = SessionManager::end_session(active_profile, start_time, end_time);
                    info!("Session recorded: {}s", duration_secs);
                }

                for (key, switch) in &active_profile.switches {
                    if switch.stats.last_session_presses > 0 {
                        info!("  {}: {} presses", key, switch.stats.last_session_presses);
                    }
                }
            }
            *was_game_running = is_game_running;
            *force_publish = true;
        }
    }
}
