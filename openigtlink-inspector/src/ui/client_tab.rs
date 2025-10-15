use eframe::egui;
use crate::types::Tab;
use super::components::{render_received_messages, render_send_panel};

pub fn render_client_tab(ui: &mut egui::Ui, tab: &mut Tab) {
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

        // Calculate heights
        let total_height = ui.available_height();
        let send_panel_height = if tab.send_panel_expanded { 300.0 } else { 50.0 };
        let messages_height = total_height - send_panel_height - 20.0;

        // Received Messages Area
        render_received_messages(
            ui,
            messages_height,
            "client_filter",
            false, // Don't show "From" column for client tab
        );

        ui.add_space(10.0);

        // Send Message Panel
        render_send_panel(
            ui,
            send_panel_height,
            &mut tab.send_panel_expanded,
            "client_quick_type",
            "client_expanded_type",
            false, // Don't show "To:" selector for client tab
            None,
        );
    });
}
