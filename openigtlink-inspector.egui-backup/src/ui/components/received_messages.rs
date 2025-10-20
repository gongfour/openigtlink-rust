use eframe::egui;
use std::collections::VecDeque;
use crate::types::ReceivedMessage;

/// Renders the received messages area with filter, search, and message table
pub fn render_received_messages(
    ui: &mut egui::Ui,
    height: f32,
    filter_id: &str,
    show_from_column: bool, // true for Server tab, false for Client tab
    messages: &VecDeque<ReceivedMessage>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_height(height);

        ui.heading("📥 Received Messages");

        ui.horizontal(|ui| {
            ui.label("Filter:");
            egui::ComboBox::from_id_salt(filter_id)
                .selected_text("All Types")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut 0, 0, "All Types");
                    ui.selectable_value(&mut 1, 1, "TRANSFORM");
                    ui.selectable_value(&mut 2, 2, "STATUS");
                });

            ui.add(egui::TextEdit::singleline(&mut String::new()).hint_text("Search..."));

            if ui.button("Clear").clicked() {
                // Clear messages
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{} messages", messages.len()));
            });
        });

        ui.separator();

        // Message list (placeholder)
        egui::ScrollArea::vertical().show(ui, |ui| {
            use egui_extras::{Column, TableBuilder};

            let table = if show_from_column {
                // Server tab: # | Time | From | Type/Device | Size
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::exact(40.0))
                    .column(Column::exact(80.0))
                    .column(Column::exact(100.0))
                    .column(Column::remainder().at_least(100.0))
                    .column(Column::exact(70.0))
            } else {
                // Client tab: # | Time | Device | Type | Size
                TableBuilder::new(ui)
                    .striped(true)
                    .column(Column::exact(40.0))
                    .column(Column::exact(80.0))
                    .column(Column::remainder().at_least(100.0))
                    .column(Column::exact(90.0))
                    .column(Column::exact(70.0))
            };

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("#");
                    });
                    header.col(|ui| {
                        ui.strong("Time");
                    });
                    if show_from_column {
                        header.col(|ui| {
                            ui.strong("From");
                        });
                    }
                    header.col(|ui| {
                        ui.strong(if show_from_column { "Type / Device" } else { "Device" });
                    });
                    if !show_from_column {
                        header.col(|ui| {
                            ui.strong("Type");
                        });
                    }
                    header.col(|ui| {
                        ui.strong("Size");
                    });
                })
                .body(|mut body| {
                    for (idx, msg) in messages.iter().enumerate() {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{}", idx + 1));
                            });
                            row.col(|ui| {
                                let time = msg
                                    .timestamp
                                    .elapsed()
                                    .map(|d| format!("{:.1}s ago", d.as_secs_f32()))
                                    .unwrap_or_else(|_| "?".to_string());
                                ui.label(time);
                            });
                            if show_from_column {
                                row.col(|ui| {
                                    ui.label(msg.from_client.as_deref().unwrap_or("-"));
                                });
                            }
                            row.col(|ui| {
                                if show_from_column {
                                    let msg_type = msg.message.message_type();
                                    let device = msg.message.device_name().unwrap_or("?");
                                    let color = egui::Color32::from_rgb(100, 200, 100);
                                    ui.colored_label(color, format!("{} / {}", msg_type, device));
                                } else {
                                    let device = msg.message.device_name().unwrap_or("?");
                                    ui.label(device);
                                }
                            });
                            if !show_from_column {
                                row.col(|ui| {
                                    let msg_type = msg.message.message_type();
                                    let color = egui::Color32::from_rgb(100, 200, 100);
                                    ui.colored_label(color, msg_type);
                                });
                            }
                            row.col(|ui| {
                                ui.label(format!("{} B", msg.size_bytes));
                            });
                        });
                    }
                });
        });

        ui.separator();
        ui.collapsing("Message Details", |ui| {
            ui.label("Selected message details here...");
        });
    });
}
