use eframe::egui;
use std::sync::Arc;
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use crate::usecase::monitor::{MonitorCommand, MonitorSharedState};
use crate::domain::models::LogicalKey;

#[derive(PartialEq)]
enum AppTab {
    Dashboard,
    Tester,
}

pub struct SwitchLifeApp {
    shared_state: Arc<ArcSwap<MonitorSharedState>>,
    command_tx: Sender<MonitorCommand>,
    current_tab: AppTab,
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
        }
    }
}

impl eframe::App for SwitchLifeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint to keep UI updated with monitor thread
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Switch Life Manager");

            // Load state (lock-free)
            let state = self.shared_state.load();

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
                ui.selectable_value(&mut self.current_tab, AppTab::Tester, "Input Tester");
            });
            ui.separator();

            match self.current_tab {
                AppTab::Dashboard => {
                    self.show_dashboard(ui, &state);
                }
                AppTab::Tester => {
                    self.show_tester(ui, &state);
                }
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.command_tx.send(MonitorCommand::Shutdown);
    }
}

impl SwitchLifeApp {
    fn show_dashboard(&self, ui: &mut egui::Ui, state: &MonitorSharedState) {
        ui.heading("Switch Statistics");

        // Sort keys for stable display
        let mut keys: Vec<_> = state.switch_stats.keys().collect();
        keys.sort_by_key(|k| k.to_string());

        egui::ScrollArea::vertical().show(ui, |ui| {
            for key in keys {
                if let Some(stats) = state.switch_stats.get(key) {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(key.to_string()).strong());
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("Total: {}", stats.total_presses));
                            });
                        });
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
}
