//! Network I/O module for OpenIGTLink communication
//!
//! Provides client and server implementations for OpenIGTLink connections
//! over both TCP (reliable) and UDP (low-latency) transports.

pub mod client;
pub mod server;
pub mod session_manager;
pub mod udp;

pub use client::IgtlClient;
pub use server::{IgtlConnection, IgtlServer};
pub use session_manager::{ClientId, ClientInfo, MessageHandler, SessionManager};
pub use udp::{UdpClient, UdpServer};
