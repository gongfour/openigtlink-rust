use eframe::egui;
use crate::connection::client_handler::run_client_connection;
use crate::types::{ConnectionCommand, Tab};
use super::components::{render_received_messages, render_send_panel};

pub fn render_client_tab(ui: &mut egui::Ui, tab: &mut Tab, runtime_handle: &tokio::runtime::Handle) {
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
                if !tab.is_connected {
                    // Create channels for communication
                    let (msg_tx, msg_rx) = std::sync::mpsc::channel();
                    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

                    tab.message_rx = Some(msg_rx);
                    tab.command_tx = Some(cmd_tx.clone());

                    let host = tab.host.clone();
                    let port = tab.port.parse::<u16>().unwrap_or(18944);

                    // Spawn background connection task
                    runtime_handle.spawn(run_client_connection(host, port, msg_tx, cmd_rx));

                    tab.is_connected = true;
                } else {
                    // Disconnect
                    if let Some(tx) = &tab.command_tx {
                        let _ = tx.send(ConnectionCommand::Disconnect);
                    }
                    tab.is_connected = false;
                }
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
            &tab.received_messages,
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
