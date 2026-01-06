use crate::app_state::AppState;
use crate::domain::models::{AppConfig, ButtonMap, LogicalKey};
use crate::usecase::monitor::{MonitorCommand, MonitorSharedState};
use chrono::{DateTime, Utc};
use tauri::State;

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
    let _ = state.command_tx.send(MonitorCommand::UpdateMapping(
        default.profile_name,
        default.bindings,
    ));
}

#[tauri::command]
pub fn set_binding(state: State<'_, AppState>, key: LogicalKey, button: u32) {
    let _ = state
        .command_tx
        .send(MonitorCommand::SetKeyBinding { key, button });
}

#[tauri::command]
pub fn reset_stats(state: State<'_, AppState>, key: LogicalKey) {
    let _ = state.command_tx.send(MonitorCommand::ResetStats { key });
}

#[tauri::command]
pub fn replace_switch(state: State<'_, AppState>, key: LogicalKey, new_model_id: String) {
    let _ = state
        .command_tx
        .send(MonitorCommand::ReplaceSwitch { key, new_model_id });
}

#[tauri::command]
pub fn set_last_replaced_date(state: State<'_, AppState>, key: LogicalKey, date: DateTime<Utc>) {
    let _ = state
        .command_tx
        .send(MonitorCommand::SetLastReplacedDate { key, date });
}

#[tauri::command]
pub async fn set_obs_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    // 1. Update Config (Persist)
    let guard = state.shared_state.load();
    let mut config = guard.config.clone();
    if config.obs_enabled == enabled {
        return Ok(()); // No change
    }
    config.obs_enabled = enabled;
    // Dispatch config update to MonitorService (persists it)
    let _ = state
        .command_tx
        .send(MonitorCommand::UpdateConfig(config.clone()));

    // 2. Control Server
    if enabled {
        state
            .obs_server
            .start(config.obs_port, state.shared_state.clone())
            .await?;
    } else {
        state.obs_server.stop().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_obs_port(state: State<'_, AppState>, port: u16) -> Result<(), String> {
    let guard = state.shared_state.load();
    let mut config = guard.config.clone();
    if config.obs_port == port {
        return Ok(());
    }
    config.obs_port = port;
    let _ = state
        .command_tx
        .send(MonitorCommand::UpdateConfig(config.clone()));

    // Restart if running
    if state.obs_server.is_running().await {
        state.obs_server.stop().await;
        // Allow some time for port release? Usually immediate.
        state
            .obs_server
            .start(port, state.shared_state.clone())
            .await?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_obs_poll_interval(state: State<'_, AppState>, interval_ms: u64) {
    let guard = state.shared_state.load();
    let mut config = guard.config.clone();
    config.obs_poll_interval_ms = interval_ms;
    let _ = state.command_tx.send(MonitorCommand::UpdateConfig(config));
}

#[tauri::command]
pub async fn get_obs_status(state: State<'_, AppState>) -> Result<String, String> {
    if state.obs_server.is_running().await {
        Ok("Running".to_string())
    } else {
        Ok("Stopped".to_string())
    }
}
