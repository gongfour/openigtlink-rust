//! OpenIGTLink Server Example
//!
//! This example demonstrates how to create a simple OpenIGTLink server
//! that accepts client connections and handles different message types.
//!
//! # Usage
//!
//! ```bash
//! # Start server on default port 18944
//! cargo run --example server
//!
//! # Start server on custom port
//! cargo run --example server 12345
//! ```

use openigtlink_rust::error::Result;
use openigtlink_rust::io::{IgtlConnection, IgtlServer};
use std::env;

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERROR] {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse port from command line arguments (default: 18944)
    let port = env::args()
        .nth(1)
        .unwrap_or_else(|| "18944".to_string());

    let addr = format!("127.0.0.1:{}", port);

    // Bind server to address
    let server = IgtlServer::bind(&addr)?;
    println!("Server listening on {}", addr);
    println!("Press Ctrl+C to stop\n");

    // Accept client connections in a loop
    loop {
        match server.accept() {
            Ok(conn) => {
                if let Err(e) = handle_client(conn) {
                    eprintln!("[ERROR] Client handler failed: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to accept connection: {}", e);
            }
        }
    }
}

fn handle_client(conn: IgtlConnection) -> Result<()> {
    println!("Client connected: {}", conn.peer_addr()?);
    Ok(())
}
