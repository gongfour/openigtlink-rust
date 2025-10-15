use eframe::egui;

/// Renders the received messages area with filter, search, and message table
pub fn render_received_messages(
    ui: &mut egui::Ui,
    height: f32,
    filter_id: &str,
    show_from_column: bool, // true for Server tab, false for Client tab
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
                ui.label("145 messages");
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
                    for i in 0..10 {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{}", i + 1));
                            });
                            row.col(|ui| {
                                ui.label("10:23:45");
                            });
                            if show_from_column {
                                row.col(|ui| {
                                    ui.label(format!("Client-{}", (i % 2) + 1));
                                });
                            }
                            row.col(|ui| {
                                let color = egui::Color32::from_rgb(100, 200, 100);
                                if show_from_column {
                                    ui.colored_label(color, "TRANSFORM / TestDevice");
                                } else {
                                    ui.label("Tool01");
                                }
                            });
                            if !show_from_column {
                                row.col(|ui| {
                                    let color = egui::Color32::from_rgb(100, 200, 100);
                                    ui.colored_label(color, "TRANSFORM");
                                });
                            }
                            row.col(|ui| {
                                ui.label("72 B");
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
