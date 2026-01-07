use crate::domain::repositories::SessionRepository;
use crate::infrastructure::obs_server::ObsServer;
use crate::usecase::monitor::{MonitorCommand, MonitorSharedState};
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use std::sync::Arc;

pub struct AppState {
    pub shared_state: Arc<ArcSwap<MonitorSharedState>>,
    pub command_tx: Sender<MonitorCommand>,
    pub obs_server: Arc<ObsServer>,
    pub session_repository: Arc<dyn SessionRepository>,
}

impl AppState {
    pub fn new(
        shared_state: Arc<ArcSwap<MonitorSharedState>>,
        command_tx: Sender<MonitorCommand>,
        obs_server: Arc<ObsServer>,
        session_repository: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            shared_state,
            command_tx,
            obs_server,
            session_repository,
        }
    }
}
