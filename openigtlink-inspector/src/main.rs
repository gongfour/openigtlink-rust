use eframe::egui;

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

#[derive(Debug, Clone, PartialEq)]
enum TabType {
    Client,
    Server,
}

#[derive(Debug, Clone)]
struct Tab {
    id: usize,
    name: String,
    tab_type: TabType,
    host: String, // Client only
    port: String,
    is_connected: bool,
    send_panel_expanded: bool,
}

impl Tab {
    fn new_client(id: usize) -> Self {
        Self {
            id,
            name: format!("Client-{}", id),
            tab_type: TabType::Client,
            host: "127.0.0.1".to_string(),
            port: "18944".to_string(),
            is_connected: false,
            send_panel_expanded: false,
        }
    }

    fn new_server(id: usize) -> Self {
        Self {
            id,
            name: format!("Server-{}", id),
            tab_type: TabType::Server,
            host: String::new(),
            port: "18944".to_string(),
            is_connected: false,
            send_panel_expanded: false,
        }
    }
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
                let tab_type = self.tabs[self.active_tab].tab_type.clone();
                match tab_type {
                    TabType::Client => self.render_client_tab(ui),
                    TabType::Server => self.render_server_tab(ui),
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

impl InspectorApp {
    fn render_client_tab(&mut self, ui: &mut egui::Ui) {
        let tab = &mut self.tabs[self.active_tab];

        ui.vertical(|ui| {
            // Connection controls
            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.text_edit_singleline(&mut tab.host);
                ui.label("Port:");
                ui.add(egui::TextEdit::singleline(&mut tab.port).desired_width(60.0));

                let button_text = if tab.is_connected {
                    "Disconnect"
                } else {
                    "Connect"
                };
                if ui.button(button_text).clicked() {
                    tab.is_connected = !tab.is_connected;
                }

                if tab.is_connected {
                    ui.colored_label(egui::Color32::GREEN, "● Connected");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "○ Disconnected");
                }
            });

            ui.separator();

            // Main area: Received Messages (80%)
            let total_height = ui.available_height();
            let send_panel_height = if tab.send_panel_expanded { 300.0 } else { 50.0 };
            let messages_height = total_height - send_panel_height - 20.0;

            // Received Messages Area
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_height(messages_height);

                ui.heading("📥 Received Messages");

                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    egui::ComboBox::from_id_salt("client_filter")
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

                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::exact(40.0))
                        .column(Column::exact(80.0))
                        .column(Column::remainder().at_least(100.0))
                        .column(Column::exact(90.0))
                        .column(Column::exact(70.0))
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("#");
                            });
                            header.col(|ui| {
                                ui.strong("Time");
                            });
                            header.col(|ui| {
                                ui.strong("Device");
                            });
                            header.col(|ui| {
                                ui.strong("Type");
                            });
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
                                    row.col(|ui| {
                                        ui.label("Tool01");
                                    });
                                    row.col(|ui| {
                                        let color = egui::Color32::from_rgb(100, 200, 100);
                                        ui.colored_label(color, "TRANSFORM");
                                    });
                                    row.col(|ui| {
                                        ui.label("72 B");
                                    });
                                });
                            }
                        });
                });

                ui.separator();

                // Message Details (collapsible)
                ui.collapsing("Message Details (Selected: #1)", |ui| {
                    ui.label("Device: Tool01");
                    ui.label("Type: TRANSFORM");
                    ui.label("Timestamp: 1234567890");
                    ui.label("Size: 72 bytes");
                });
            });

            ui.add_space(10.0);

            // Send Message Panel (collapsible)
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let arrow = if tab.send_panel_expanded {
                        "▼"
                    } else {
                        "▶"
                    };
                    if ui.button(format!("{} Send Message", arrow)).clicked() {
                        tab.send_panel_expanded = !tab.send_panel_expanded;
                    }

                    if !tab.send_panel_expanded {
                        ui.label("Type:");
                        egui::ComboBox::from_id_salt("quick_send_type")
                            .selected_text("TRANSFORM")
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.label("TRANSFORM");
                            });

                        ui.add(
                            egui::TextEdit::singleline(&mut String::from("TestDevice"))
                                .desired_width(150.0),
                        );

                        if ui.button("Send").clicked() {
                            // Send message
                        }
                    }
                });

                if tab.send_panel_expanded {
                    ui.separator();
                    ui.label("Message Type:");
                    egui::ComboBox::from_id_salt("expanded_send_type")
                        .selected_text("TRANSFORM")
                        .show_ui(ui, |ui| {
                            ui.label("TRANSFORM");
                            ui.label("STATUS");
                            ui.label("STRING");
                        });

                    ui.label("Device Name:");
                    ui.text_edit_singleline(&mut String::from("TestDevice"));

                    ui.separator();
                    ui.label("📝 Content Editor (Type-specific)");
                    ui.label("[Matrix editor placeholder]");

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Send Once").clicked() {
                            // Send
                        }
                        if ui.button("Send @ 60Hz").clicked() {
                            // Start repeat
                        }
                        if ui.button("Stop").clicked() {
                            // Stop repeat
                        }
                    });
                }
            });
        });
    }

    fn render_server_tab(&mut self, ui: &mut egui::Ui) {
        let tab = &mut self.tabs[self.active_tab];

        ui.vertical(|ui| {
            // Server controls
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.add(egui::TextEdit::singleline(&mut tab.port).desired_width(60.0));

                let button_text = if tab.is_connected { "Stop" } else { "Listen" };
                if ui.button(button_text).clicked() {
                    tab.is_connected = !tab.is_connected;
                }

                if tab.is_connected {
                    ui.colored_label(egui::Color32::GREEN, "● Listening");
                } else {
                    ui.colored_label(egui::Color32::GRAY, "○ Not listening");
                }

                ui.separator();

                // Client filter selector
                ui.label("View:");
                egui::ComboBox::from_id_salt("client_selector")
                    .selected_text("All Clients")
                    .show_ui(ui, |ui| {
                        let _ = ui.selectable_label(true, "All Clients");
                        let _ = ui.selectable_label(false, "Client-1 (192.168.1.101)");
                        let _ = ui.selectable_label(false, "Client-2 (192.168.1.102)");
                    });
            });

            ui.separator();

            // Main area: Received Messages (80%)
            let total_height = ui.available_height();
            let send_panel_height = if tab.send_panel_expanded { 300.0 } else { 50.0 };
            let messages_height = total_height - send_panel_height - 20.0;

            // Received Messages Area
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_height(messages_height);

                ui.heading("📥 Received Messages");

                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    egui::ComboBox::from_id_salt("server_filter")
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

                    TableBuilder::new(ui)
                        .striped(true)
                        .column(Column::exact(40.0))
                        .column(Column::exact(80.0))
                        .column(Column::exact(100.0))
                        .column(Column::remainder().at_least(100.0))
                        .column(Column::exact(70.0))
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("#");
                            });
                            header.col(|ui| {
                                ui.strong("Time");
                            });
                            header.col(|ui| {
                                ui.strong("From");
                            });
                            header.col(|ui| {
                                ui.strong("Type / Device");
                            });
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
                                    row.col(|ui| {
                                        ui.label(format!("Client-{}", (i % 2) + 1));
                                    });
                                    row.col(|ui| {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(100, 200, 100),
                                            "TRANSFORM / TestDevice",
                                        );
                                    });
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

            ui.add_space(10.0);

            // Send Message Panel
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_height(send_panel_height);

                ui.horizontal(|ui| {
                    let arrow = if tab.send_panel_expanded { "▼" } else { "▶" };
                    if ui.button(format!("{} Send Message", arrow)).clicked() {
                        tab.send_panel_expanded = !tab.send_panel_expanded;
                    }

                    if !tab.send_panel_expanded {
                        ui.label("Type:");
                        egui::ComboBox::from_id_salt("server_quick_type")
                            .selected_text("TRANSFORM")
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                ui.label("TRANSFORM");
                                ui.label("STATUS");
                                ui.label("STRING");
                            });

                        ui.add(
                            egui::TextEdit::singleline(&mut String::from("TestDevice"))
                                .desired_width(120.0),
                        );

                        ui.label("To:");
                        egui::ComboBox::from_id_salt("server_quick_to")
                            .selected_text("All Clients")
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                ui.label("All Clients");
                                ui.label("Client-1");
                                ui.label("Client-2");
                            });

                        if ui.button("Send").clicked() {
                            // Send message
                        }
                    }
                });

                if tab.send_panel_expanded {
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Send to:");
                        egui::ComboBox::from_id_salt("server_expanded_to")
                            .selected_text("All Clients")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut 0, 0, "All Clients");
                                ui.selectable_value(&mut 1, 1, "Client-1");
                                ui.selectable_value(&mut 2, 2, "Client-2");
                            });
                    });

                    ui.separator();

                    ui.label("Message Type:");
                    egui::ComboBox::from_id_salt("server_expanded_type")
                        .selected_text("TRANSFORM")
                        .show_ui(ui, |ui| {
                            ui.label("TRANSFORM");
                            ui.label("STATUS");
                            ui.label("STRING");
                        });

                    ui.label("Device Name:");
                    ui.text_edit_singleline(&mut String::from("TestDevice"));

                    ui.separator();
                    ui.label("📝 Content Editor (Type-specific)");
                    ui.label("[Matrix editor placeholder]");

                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Send Once").clicked() {
                            // Send
                        }
                        if ui.button("Send @ 60Hz").clicked() {
                            // Start repeat
                        }
                        if ui.button("Stop").clicked() {
                            // Stop repeat
                        }
                    });
                }
            });
        });
    }
}
