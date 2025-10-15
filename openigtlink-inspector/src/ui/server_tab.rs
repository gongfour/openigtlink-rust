use eframe::egui;
use crate::types::Tab;
use super::components::{render_received_messages, render_send_panel};

pub fn render_server_tab(ui: &mut egui::Ui, tab: &mut Tab) {
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

        // Calculate heights
        let total_height = ui.available_height();
        let send_panel_height = if tab.send_panel_expanded { 300.0 } else { 50.0 };
        let messages_height = total_height - send_panel_height - 20.0;

        // Received Messages Area
        render_received_messages(
            ui,
            messages_height,
            "server_filter",
            true, // Show "From" column for server tab
            &tab.received_messages,
        );

        ui.add_space(10.0);

        // Send Message Panel
        render_send_panel(
            ui,
            send_panel_height,
            &mut tab.send_panel_expanded,
            "server_quick_type",
            "server_expanded_type",
            true,                    // Show "To:" selector for server tab
            Some("server_to"),
        );
    });
}
