use eframe::egui;

/// Renders the send message panel (collapsible)
pub fn render_send_panel(
    ui: &mut egui::Ui,
    height: f32,
    is_expanded: &mut bool,
    quick_type_id: &str,
    expanded_type_id: &str,
    show_to_selector: bool, // true for Server tab
    to_selector_id: Option<&str>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_height(height);

        ui.horizontal(|ui| {
            let arrow = if *is_expanded { "▼" } else { "▶" };
            if ui
                .button(format!("{} Send Message", arrow))
                .clicked()
            {
                *is_expanded = !*is_expanded;
            }

            if !*is_expanded {
                ui.label("Type:");
                egui::ComboBox::from_id_salt(quick_type_id)
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

                // Show "To:" selector for Server tab
                if show_to_selector {
                    if let Some(to_id) = to_selector_id {
                        ui.label("To:");
                        egui::ComboBox::from_id_salt(to_id)
                            .selected_text("All Clients")
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                ui.label("All Clients");
                                ui.label("Client-1");
                                ui.label("Client-2");
                            });
                    }
                }

                if ui.button("Send").clicked() {
                    // Send message
                }
            }
        });

        if *is_expanded {
            ui.separator();

            // Show "Send to:" selector for Server tab in expanded mode
            if show_to_selector {
                if let Some(to_id) = to_selector_id {
                    ui.horizontal(|ui| {
                        ui.label("Send to:");
                        egui::ComboBox::from_id_salt(format!("{}_expanded", to_id))
                            .selected_text("All Clients")
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut 0, 0, "All Clients");
                                ui.selectable_value(&mut 1, 1, "Client-1");
                                ui.selectable_value(&mut 2, 2, "Client-2");
                            });
                    });

                    ui.separator();
                }
            }

            ui.label("Message Type:");
            egui::ComboBox::from_id_salt(expanded_type_id)
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
}
