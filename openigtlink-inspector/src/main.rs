use eframe::egui;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OpenIGTLink Inspector",
        options,
        Box::new(|_cc| Ok(Box::new(InspectorApp::default()))),
    )
}

#[derive(Default)]
struct InspectorApp {
    // Connection settings
    host: String,
    port: String,
    is_connected: bool,

    // Message list
    filter_text: String,

    // Settings
    show_settings: bool,
}

impl eframe::App for InspectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔍 OpenIGTLink Inspector");
                ui.separator();

                ui.label("Host:");
                ui.text_edit_singleline(&mut self.host);

                ui.label("Port:");
                ui.add(egui::TextEdit::singleline(&mut self.port).desired_width(60.0));

                let connect_text = if self.is_connected {
                    "Disconnect"
                } else {
                    "Connect"
                };

                if ui.button(connect_text).clicked() {
                    self.is_connected = !self.is_connected;
                }

                ui.separator();

                // Status indicator
                if self.is_connected {
                    ui.colored_label(egui::Color32::GREEN, "● Connected");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "○ Disconnected");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }

                    if ui.button("🌙 Dark").clicked() {
                        // Toggle theme (will implement later)
                    }
                });
            });
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Messages: 0");
                ui.separator();
                ui.label("Bytes: 0");
                ui.separator();
                ui.label("Connected: 00:00");
            });
        });

        // Left panel - Message list
        egui::SidePanel::left("message_list")
            .default_width(500.0)
            .min_width(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Messages");

                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    egui::ComboBox::from_label("")
                        .selected_text("All Types")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut 0, 0, "All Types");
                            ui.selectable_value(&mut 1, 1, "TRANSFORM");
                            ui.selectable_value(&mut 2, 2, "STATUS");
                            ui.selectable_value(&mut 3, 3, "STRING");
                        });

                    ui.add(
                        egui::TextEdit::singleline(&mut self.filter_text)
                            .hint_text("Search...")
                            .desired_width(150.0)
                    );
                });

                ui.separator();

                // Message table
                use egui_extras::{Column, TableBuilder};

                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::exact(40.0)) // #
                    .column(Column::exact(80.0)) // Time
                    .column(Column::remainder().at_least(100.0)) // Device
                    .column(Column::exact(90.0)) // Type
                    .column(Column::exact(70.0)) // Size
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.strong("#"); });
                        header.col(|ui| { ui.strong("Time"); });
                        header.col(|ui| { ui.strong("Device"); });
                        header.col(|ui| { ui.strong("Type"); });
                        header.col(|ui| { ui.strong("Size"); });
                    })
                    .body(|mut body| {
                        // Sample data for demo
                        for i in 0..20 {
                            body.row(18.0, |mut row| {
                                row.col(|ui| { ui.label(format!("{}", i + 1)); });
                                row.col(|ui| { ui.label("10:23:45"); });
                                row.col(|ui| { ui.label("Tool01"); });
                                row.col(|ui| {
                                    let color = match i % 3 {
                                        0 => egui::Color32::from_rgb(100, 200, 100),
                                        1 => egui::Color32::from_rgb(100, 150, 200),
                                        _ => egui::Color32::from_rgb(200, 200, 100),
                                    };
                                    ui.colored_label(color, "TRANSFORM");
                                });
                                row.col(|ui| { ui.label("72 B"); });
                            });
                        }
                    });

                ui.separator();

                if ui.button("🗑 Clear All").clicked() {
                    // Clear messages
                }
            });

        // Central panel - Message details
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Message Details");
            ui.separator();

            ui.horizontal(|ui| {
                let _ = ui.selectable_label(true, "📋 Content");
                let _ = ui.selectable_label(false, "🔢 Raw Hex");
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.group(|ui| {
                    ui.label("Header");
                    ui.separator();

                    egui::Grid::new("header_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Device Name:");
                            ui.label("Tool01");
                            ui.end_row();

                            ui.label("Type:");
                            ui.label("TRANSFORM");
                            ui.end_row();

                            ui.label("Timestamp:");
                            ui.label("1234567890");
                            ui.end_row();

                            ui.label("Body Size:");
                            ui.label("72 bytes");
                            ui.end_row();
                        });
                });

                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.label("Content - 4x4 Transformation Matrix");
                    ui.separator();

                    egui::Grid::new("matrix_grid")
                        .num_columns(4)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            for row in 0..4 {
                                for col in 0..4 {
                                    let val = if row == col { 1.0 } else { 0.0 };
                                    ui.monospace(format!("{:8.3}", val));
                                }
                                ui.end_row();
                            }
                        });
                });
            });
        });

        // Settings window (modal)
        egui::Window::new("⚙ Settings")
            .open(&mut self.show_settings)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Appearance");
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    egui::ComboBox::from_label("")
                        .selected_text("Dark")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut 0, 0, "Dark");
                            ui.selectable_value(&mut 1, 1, "Light");
                        });
                });

                ui.separator();

                ui.heading("Connection");
                ui.label("Default Host: 127.0.0.1");
                ui.label("Default Port: 18944");

                ui.separator();

                ui.heading("Messages");
                ui.label("Buffer Size: 1000");
                ui.checkbox(&mut true, "Auto-scroll to latest");

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        // Settings saved
                    }
                    if ui.button("Cancel").clicked() {
                        // Cancelled
                    }
                });
            });
    }
}
