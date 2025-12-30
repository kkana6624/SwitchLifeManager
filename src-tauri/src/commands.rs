use tauri::State;
use crate::app_state::AppState;
use crate::usecase::monitor::{MonitorCommand, MonitorSharedState};
use crate::domain::models::{LogicalKey, AppConfig, ButtonMap};

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> MonitorSharedState {
    let guard = state.shared_state.load();
    (**guard).clone()
}

#[tauri::command]
pub fn force_save(state: State<'_, AppState>) {
    let _ = state.command_tx.send(MonitorCommand::ForceSave);
}

#[tauri::command]
pub fn update_config(state: State<'_, AppState>, config: AppConfig) {
    let _ = state.command_tx.send(MonitorCommand::UpdateConfig(config));
}

#[tauri::command]
pub fn set_target_controller(state: State<'_, AppState>, index: u32) {
    let guard = state.shared_state.load();
    let mut config = guard.config.clone();
    config.target_controller_index = index;
    let _ = state.command_tx.send(MonitorCommand::UpdateConfig(config));
}

#[tauri::command]
pub fn reset_to_default_mapping(state: State<'_, AppState>) {
    let default = ButtonMap::default();
    let _ = state.command_tx.send(MonitorCommand::UpdateMapping(default.profile_name, default.bindings));
}

#[tauri::command]
pub fn set_binding(state: State<'_, AppState>, key: LogicalKey, button: u16) {
    let _ = state.command_tx.send(MonitorCommand::SetKeyBinding { key, button });
}

#[tauri::command]
pub fn reset_stats(state: State<'_, AppState>, key: LogicalKey) {
    let _ = state.command_tx.send(MonitorCommand::ResetStats { key });
}

#[tauri::command]
pub fn replace_switch(state: State<'_, AppState>, key: LogicalKey, new_model_id: String) {
    let _ = state.command_tx.send(MonitorCommand::ReplaceSwitch { key, new_model_id });
}
