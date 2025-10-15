use eframe::egui;

mod connection;
mod types;
mod ui;

use types::{Tab, TabType};
use ui::{render_client_tab, render_server_tab};

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "OpenIGTLink Inspector",
        options,
        Box::new(|_cc| Ok(Box::new(InspectorApp::default()))),
    )
}

struct InspectorApp {
    tabs: Vec<Tab>,
    active_tab: usize,
    next_tab_id: usize,
    show_new_tab_dialog: bool,
    show_settings: bool,
}

impl Default for InspectorApp {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::new_client(0)],
            active_tab: 0,
            next_tab_id: 1,
            show_new_tab_dialog: false,
            show_settings: false,
        }
    }
}

impl eframe::App for InspectorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top panel - App title and controls
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔍 OpenIGTLink Inspector");
                ui.separator();

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⚙ Settings").clicked() {
                        self.show_settings = !self.show_settings;
                    }
                    if ui.button("🌙 Theme").clicked() {
                        // Theme toggle
                    }
                });
            });
        });

        // Tab bar
        egui::TopBottomPanel::top("tab_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut tab_to_close = None;

                for (idx, tab) in self.tabs.iter().enumerate() {
                    let icon = match tab.tab_type {
                        TabType::Client => "📡",
                        TabType::Server => "🏠",
                    };

                    let status_color = if tab.is_connected {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::GRAY
                    };

                    ui.horizontal(|ui| {
                        let button_text = format!("{} {}", icon, tab.name);
                        let button =
                            egui::Button::new(button_text).fill(if idx == self.active_tab {
                                ui.style().visuals.selection.bg_fill
                            } else {
                                egui::Color32::TRANSPARENT
                            });

                        if ui.add(button).clicked() {
                            self.active_tab = idx;
                        }

                        ui.colored_label(status_color, "●");

                        if ui.small_button("×").clicked() {
                            tab_to_close = Some(idx);
                        }
                    });
                }

                if let Some(idx) = tab_to_close {
                    self.tabs.remove(idx);
                    if self.active_tab >= self.tabs.len() && self.active_tab > 0 {
                        self.active_tab -= 1;
                    }
                }

                ui.separator();

                if ui.button("+ New Tab").clicked() {
                    self.show_new_tab_dialog = true;
                }
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    let status = if tab.is_connected {
                        match tab.tab_type {
                            TabType::Client => format!("Connected to {}:{}", tab.host, tab.port),
                            TabType::Server => format!("Listening on :{} | Clients: 0", tab.port),
                        }
                    } else {
                        "Disconnected".to_string()
                    };
                    ui.label(status);
                    ui.separator();
                    ui.label("Rx: 0 msgs");
                    ui.separator();
                    ui.label("Tx: 0 msgs");
                }
            });
        });

        // Main content area - Tab content
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.active_tab < self.tabs.len() {
                let tab = &mut self.tabs[self.active_tab];
                match tab.tab_type {
                    TabType::Client => render_client_tab(ui, tab),
                    TabType::Server => render_server_tab(ui, tab),
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No tabs open. Click '+ New Tab' to create one.");
                });
            }
        });

        // New tab dialog
        if self.show_new_tab_dialog {
            egui::Window::new("New Connection")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Select connection type:");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("📡 Client Connection").clicked() {
                            self.tabs.push(Tab::new_client(self.next_tab_id));
                            self.active_tab = self.tabs.len() - 1;
                            self.next_tab_id += 1;
                            self.show_new_tab_dialog = false;
                        }

                        if ui.button("🏠 Server (Listen)").clicked() {
                            self.tabs.push(Tab::new_server(self.next_tab_id));
                            self.active_tab = self.tabs.len() - 1;
                            self.next_tab_id += 1;
                            self.show_new_tab_dialog = false;
                        }
                    });

                    ui.add_space(10.0);
                    if ui.button("Cancel").clicked() {
                        self.show_new_tab_dialog = false;
                    }
                });
        }

        // Settings window
        if self.show_settings {
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

                    ui.heading("Messages");
                    ui.label("Buffer Size: 1000");
                    ui.checkbox(&mut true, "Auto-scroll to latest");
                });
        }
    }
}
