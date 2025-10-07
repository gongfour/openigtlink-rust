//! Network I/O module for OpenIGTLink communication
//!
//! Provides client and server implementations for OpenIGTLink connections
//! over both TCP (reliable) and UDP (low-latency) transports.

pub mod client;
pub mod server;
pub mod session_manager;
pub mod udp;
pub mod message_queue;
pub mod partial_transfer;

pub use client::IgtlClient;
pub use server::{IgtlConnection, IgtlServer};
pub use session_manager::{ClientId, ClientInfo, MessageHandler, SessionManager};
pub use udp::{UdpClient, UdpServer};
pub use message_queue::{MessageQueue, QueueConfig, QueueStats};
pub use partial_transfer::{
    PartialTransferManager, TransferConfig, TransferId, TransferInfo, TransferState,
};
