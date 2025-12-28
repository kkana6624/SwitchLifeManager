use eframe::egui;
use std::sync::Arc;
use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use crate::usecase::monitor::{MonitorCommand, MonitorSharedState};

pub struct SwitchLifeApp {
    shared_state: Arc<ArcSwap<MonitorSharedState>>,
    command_tx: Sender<MonitorCommand>,
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
            ui.heading("Switches");

            // Sort keys for stable display
            let mut keys: Vec<_> = state.switch_stats.keys().collect();
            // Sorting logical keys might require implementing Ord or just sort by string representation
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
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.command_tx.send(MonitorCommand::Shutdown);
    }
}
