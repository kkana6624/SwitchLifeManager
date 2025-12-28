use eframe::egui;
use std::sync::Arc;
use std::collections::HashSet;
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use crate::usecase::monitor::{MonitorCommand, MonitorSharedState};
use crate::domain::models::LogicalKey;

#[derive(PartialEq)]
enum AppTab {
    Dashboard,
    Session,
    Tester,
    Settings,
}

#[derive(Default)]
struct KeyConfigState {
    target_key: Option<LogicalKey>,
    initial_buttons: u16,
}

pub struct SwitchLifeApp {
    shared_state: Arc<ArcSwap<MonitorSharedState>>,
    command_tx: Sender<MonitorCommand>,
    current_tab: AppTab,
    key_config_state: KeyConfigState,
    bulk_selected_keys: HashSet<LogicalKey>,
}

impl SwitchLifeApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        shared_state: Arc<ArcSwap<MonitorSharedState>>,
        command_tx: Sender<MonitorCommand>,
    ) -> Self {
        // Customize fonts here if needed
        Self {
            shared_state,
            command_tx,
            current_tab: AppTab::Dashboard,
            key_config_state: KeyConfigState::default(),
            bulk_selected_keys: HashSet::new(),
        }
    }
}

impl eframe::App for SwitchLifeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint to keep UI updated with monitor thread
        ctx.request_repaint();
        
        let state = self.shared_state.load();

        // Handle Key Config Input
        if let Some(target_key) = &self.key_config_state.target_key {
            let current_raw = state.raw_button_state;
            let initial = self.key_config_state.initial_buttons;
            
            // Detect new press (current has bits that were not in initial)
            // We ignore release of initial buttons.
            let new_presses = current_raw & !initial;

            if new_presses != 0 {
                // Pick the lowest bit set (arbitrary choice for single button mapping)
                let button_bit = 1 << new_presses.trailing_zeros();
                
                let _ = self.command_tx.send(MonitorCommand::SetKeyBinding {
                    key: target_key.clone(),
                    button: button_bit,
                });
                
                // Close modal
                self.key_config_state.target_key = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Switch Life Manager");

            // Status Header
            ui.horizontal(|ui| {
                ui.label("Status: ");
                if state.is_connected {
                    ui.colored_label(egui::Color32::GREEN, "Connected");
                } else {
                    ui.colored_label(egui::Color32::RED, "Disconnected");
                }
                
                ui.separator();

                ui.label("Game: ");
                if state.is_game_running {
                     ui.colored_label(egui::Color32::BLUE, "Running");
                } else {
                     ui.label("Stopped");
                }
            });

            if let Some(msg) = &state.last_status_message {
                ui.label(format!("Log: {}", msg));
            }

            ui.separator();

            // Tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, AppTab::Dashboard, "Dashboard");
                ui.selectable_value(&mut self.current_tab, AppTab::Session, "Session Stats");
                ui.selectable_value(&mut self.current_tab, AppTab::Tester, "Input Tester");
                ui.selectable_value(&mut self.current_tab, AppTab::Settings, "Settings");
            });
            ui.separator();

            match self.current_tab {
                AppTab::Dashboard => {
                    self.show_dashboard(ui, &state);
                }
                AppTab::Session => {
                    self.show_session_stats(ui, &state);
                }
                AppTab::Tester => {
                    self.show_tester(ui, &state);
                }
                AppTab::Settings => {
                    self.show_settings(ui, &state);
                }
            }
        });

        // Modal for Key Config
        if let Some(target_key) = self.key_config_state.target_key.clone() {
            egui::Window::new("Key Configuration")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.heading(format!("Press a button for {}", target_key));
                    ui.label("Please press the button you want to assign...");
                    ui.add_space(20.0);
                    if ui.button("Cancel").clicked() {
                        self.key_config_state.target_key = None;
                    }
                });
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.command_tx.send(MonitorCommand::Shutdown);
    }
}

impl SwitchLifeApp {
    fn show_session_stats(&self, ui: &mut egui::Ui, state: &MonitorSharedState) {
        ui.heading("Session Statistics");
        
        // Status with better visibility on light backgrounds
        if state.is_game_running {
             ui.colored_label(egui::Color32::from_rgb(0, 150, 0), "● Game is Running - Recording Stats...");
        } else {
             ui.colored_label(egui::Color32::from_rgb(0, 100, 200), "■ Game Stopped - Showing Last Session Result");
        }
        ui.add_space(10.0);

        let mut keys: Vec<_> = state.switches.keys().collect();
        keys.sort_by_key(|k| k.to_string());
        
        let mut session_total_presses = 0;
        let mut session_total_chatters = 0;

        // Calculate totals for session
        for key in &keys {
            if let Some(switch) = state.switches.get(key) {
                session_total_presses += switch.stats.last_session_presses;
                session_total_chatters += switch.stats.last_session_chatters;
            }
        }

        // Summary
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Session Presses: {}", session_total_presses)).strong().size(18.0));
                ui.add_space(20.0);
                
                let chatter_text = format!("Session Chatters: {}", session_total_chatters);
                if session_total_chatters > 0 {
                    ui.label(egui::RichText::new(chatter_text).color(egui::Color32::RED).strong().size(18.0));
                } else {
                    ui.label(egui::RichText::new(chatter_text).strong().size(18.0));
                }
            });
        });
        ui.add_space(10.0);

        egui::Grid::new("session_stats_grid").striped(true).spacing([40.0, 10.0]).show(ui, |ui| {
            ui.label(egui::RichText::new("Key").strong());
            ui.label(egui::RichText::new("Presses").strong());
            ui.label(egui::RichText::new("Chatters").strong());
            ui.label(egui::RichText::new("Rate").strong());
            ui.end_row();

            for key in keys {
                if let Some(switch) = state.switches.get(key) {
                    let stats = &switch.stats;
                    
                    ui.label(key.to_string());
                    ui.label(format!("{}", stats.last_session_presses));
                    
                    if stats.last_session_chatters > 0 {
                        ui.colored_label(egui::Color32::RED, format!("{}", stats.last_session_chatters));
                    } else {
                        ui.label("0");
                    }

                    let rate = if stats.last_session_presses > 0 {
                        (stats.last_session_chatters as f64 / (stats.last_session_presses + stats.last_session_chatters) as f64) * 100.0
                    } else {
                        0.0
                    };
                    
                    let rate_text = format!("{:.2}%", rate);
                    if rate > 1.0 {
                         ui.colored_label(egui::Color32::RED, rate_text);
                    } else if rate > 0.0 {
                         ui.colored_label(egui::Color32::from_rgb(180, 120, 0), rate_text); // Darker Gold/Orange
                    } else {
                         ui.label(rate_text);
                    }
                    
                    ui.end_row();
                }
            }
        });
    }

    fn show_dashboard(&mut self, ui: &mut egui::Ui, state: &MonitorSharedState) {
        ui.heading("Switch Statistics");

        // Sort keys for stable display
        let mut keys: Vec<_> = state.switches.keys().collect();
        keys.sort_by_key(|k| k.to_string());

        // Helper to get rated lifespan
        let default_models = crate::domain::models::get_default_switch_models();

        // --- Bulk Action Area ---
        ui.group(|ui| {
            ui.heading("Bulk Actions");
            ui.horizontal(|ui| {
                if ui.button("Select All").clicked() {
                    for k in &keys {
                        self.bulk_selected_keys.insert((*k).clone());
                    }
                }
                if ui.button("Deselect All").clicked() {
                    self.bulk_selected_keys.clear();
                }
            });
            
            let mut bulk_model_id = ui.data_mut(|d| d.get_temp::<String>(egui::Id::new("bulk_model_id")))
                .unwrap_or_else(|| "omron_d2mv_01_1c3".to_string());
            
            ui.horizontal(|ui| {
                 let model_name = default_models.iter().find(|m| m.id == bulk_model_id).map(|m| m.name.clone()).unwrap_or("Unknown".to_string());
                 
                 ui.label("Model:");
                 egui::ComboBox::from_id_salt("bulk_combo")
                    .selected_text(model_name)
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        for model in &default_models {
                            ui.selectable_value(&mut bulk_model_id, model.id.clone(), &model.name);
                        }
                    });
                 
                 if ui.button("Apply to Selected").clicked() {
                     for key in &self.bulk_selected_keys {
                         let _ = self.command_tx.send(MonitorCommand::ReplaceSwitch {
                             key: key.clone(),
                             new_model_id: bulk_model_id.clone(),
                         });
                     }
                 }
            });
            ui.label(egui::RichText::new("Note: Applying model resets stats for selected keys.").small().italics());
            
            ui.data_mut(|d| d.insert_temp(egui::Id::new("bulk_model_id"), bulk_model_id));
        });
        ui.add_space(10.0);
        // ------------------------

        egui::ScrollArea::vertical().show(ui, |ui| {
            for key in keys {
                if let Some(switch_data) = state.switches.get(key) {
                    let stats = &switch_data.stats;

                    // Find model info
                    let model_info = default_models.iter()
                        .find(|m| m.id == switch_data.switch_model_id)
                        .or_else(|| default_models.iter().find(|m| m.id == "generic_unknown"));
                    
                    let rated_lifespan = model_info.map(|m| m.rated_lifespan_presses).unwrap_or(1_000_000) as f64;
                    
                    // Calculate Life
                    let usage_ratio = stats.total_presses as f64 / rated_lifespan;
                    let remaining_ratio = 1.0 - usage_ratio;
                    
                    // Determine Color
                    let bar_color = if remaining_ratio > 0.50 {
                         egui::Color32::GREEN
                    } else if remaining_ratio > 0.25 {
                         egui::Color32::YELLOW
                    } else {
                         egui::Color32::RED
                    };
                    
                    let display_ratio = remaining_ratio.clamp(0.0, 1.0) as f32;

                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let mut is_selected = self.bulk_selected_keys.contains(key);
                            if ui.checkbox(&mut is_selected, "").changed() {
                                if is_selected { self.bulk_selected_keys.insert(key.clone()); }
                                else { self.bulk_selected_keys.remove(key); }
                            }
                            
                            ui.label(egui::RichText::new(key.to_string()).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("Total: {}", stats.total_presses));
                            });
                        });
                        
                        // Life Bar
                        ui.add_space(2.0);
                        ui.vertical(|ui| {
                            let bar = egui::ProgressBar::new(display_ratio)
                                .text(format!("Life: {:.1}% ({})", remaining_ratio * 100.0, switch_data.switch_model_id))
                                .fill(bar_color);
                            ui.add(bar);
                        });
                        ui.add_space(5.0);

                        // Switch Management Controls
                        ui.horizontal(|ui| {
                            ui.label("Switch Model:");
                            
                            let mut selected_model_id = switch_data.switch_model_id.clone();
                            let current_model_name = model_info.map(|m| m.name.clone()).unwrap_or_else(|| "Unknown".to_string());
                            
                            egui::ComboBox::from_id_salt(format!("combo_{}", key))
                                .selected_text(current_model_name)
                                .width(200.0)
                                .show_ui(ui, |ui| {
                                    for model in &default_models {
                                        ui.selectable_value(&mut selected_model_id, model.id.clone(), &model.name);
                                    }
                                });
                            
                            if selected_model_id != switch_data.switch_model_id {
                                let _ = self.command_tx.send(MonitorCommand::ReplaceSwitch {
                                    key: key.clone(),
                                    new_model_id: selected_model_id,
                                });
                            }

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                 if ui.button("Reset Stats").clicked() {
                                     let _ = self.command_tx.send(MonitorCommand::ResetStats { key: key.clone() });
                                 }
                            });
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.label(format!("Session: {}", stats.last_session_presses));
                            ui.add_space(10.0);
                            ui.label(format!("Chatters: {} ({}%)", 
                                stats.total_chatters,
                                if stats.total_presses > 0 {
                                    (stats.total_chatters as f64 / (stats.total_presses + stats.total_chatters) as f64 * 100.0) as u64
                                } else { 0 }
                            ));
                        });
                    });
                }
            }
        });
    }

    fn show_tester(&self, ui: &mut egui::Ui, state: &MonitorSharedState) {
        ui.heading("Real-time Input Tester");
        ui.label("Press buttons on your controller to see them light up.");

        if !state.is_connected {
            ui.colored_label(egui::Color32::RED, "Controller Disconnected - Please connect your device.");
            return;
        }

        // We iterate over known bindings to display buttons in a logical order if possible
        let mut sorted_keys: Vec<_> = state.bindings.keys().collect();
        sorted_keys.sort_by_key(|k| k.to_string());

        egui::Grid::new("tester_grid").striped(true).spacing([20.0, 10.0]).show(ui, |ui| {
            for key in sorted_keys {
                let is_pressed = state.current_pressed_keys.contains(key);
                
                // Label
                ui.label(egui::RichText::new(key.to_string()).size(16.0));

                // Indicator
                let (color, text) = if is_pressed {
                    (egui::Color32::GREEN, "ON")
                } else {
                    (egui::Color32::GRAY, "OFF")
                };
                
                // Draw a simple colored box/label
                ui.add(egui::Label::new(
                    egui::RichText::new(text).color(egui::Color32::WHITE).background_color(color).heading()
                ));

                ui.end_row();
            }
        });

        ui.add_space(20.0);
        ui.label(egui::RichText::new("Note: This view shows raw input state derived from your key mappings.").italics());
    }

    fn show_settings(&mut self, ui: &mut egui::Ui, state: &MonitorSharedState) {
        ui.heading("Configuration");
        
        ui.group(|ui| {
             ui.heading("Key Bindings");
             ui.label("Click 'Set' then press a button on your controller.");
             
             let mut sorted_keys: Vec<_> = state.bindings.keys().collect();
             sorted_keys.sort_by_key(|k| k.to_string());

             egui::Grid::new("settings_grid").striped(true).spacing([20.0, 10.0]).show(ui, |ui| {
                for key in sorted_keys {
                    ui.label(key.to_string());
                    
                    let mask = state.bindings.get(key).unwrap_or(&0);
                    ui.monospace(format!("0x{:04X}", mask));

                    if ui.button("Set").clicked() {
                        self.key_config_state.target_key = Some(key.clone());
                        self.key_config_state.initial_buttons = state.raw_button_state;
                    }
                    ui.end_row();
                }
             });
        });

        ui.add_space(20.0);
        ui.horizontal(|ui| {
            if ui.button("Save Configuration Forcefully").clicked() {
                let _ = self.command_tx.send(MonitorCommand::ForceSave);
            }
            if ui.button("Reset to Default Mapping").clicked() {
                let default_map = crate::domain::models::ButtonMap::default();
                let _ = self.command_tx.send(MonitorCommand::UpdateMapping(
                    default_map.profile_name,
                    default_map.bindings,
                ));
            }
        });
    }
}