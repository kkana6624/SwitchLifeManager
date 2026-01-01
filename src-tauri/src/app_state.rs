use std::sync::Arc;
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use crate::usecase::monitor::{MonitorCommand, MonitorSharedState};

pub struct AppState {
    pub shared_state: Arc<ArcSwap<MonitorSharedState>>,
    pub command_tx: Sender<MonitorCommand>,
}

impl AppState {
    pub fn new(shared_state: Arc<ArcSwap<MonitorSharedState>>, command_tx: Sender<MonitorCommand>) -> Self {
        Self {
            shared_state,
            command_tx,
        }
    }
}
