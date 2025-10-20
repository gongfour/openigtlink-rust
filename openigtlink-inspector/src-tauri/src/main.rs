// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod connection;
mod types;

use connection::ConnectionManager;
use std::sync::Mutex;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let context = tauri::generate_context!();

    tauri::Builder::default()
        .manage(Mutex::new(ConnectionManager::new()))
        .invoke_handler(tauri::generate_handler![
            commands::connect_client,
            commands::disconnect_client,
            commands::get_connection_status,
            commands::create_client_tab,
            commands::create_server_tab,
        ])
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .run(context)
        .expect("error while running tauri application");
}
