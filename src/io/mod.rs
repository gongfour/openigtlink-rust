//! Network I/O module for OpenIGTLink communication
//!
//! Provides client and server implementations for OpenIGTLink connections
//! over both TCP (reliable) and UDP (low-latency) transports.

mod common;
pub mod unified_client;
pub mod tls_reconnect;
pub mod builder;
pub mod client;
pub mod server;
pub mod async_client;
pub mod async_server;
pub mod tls_client;
pub mod tls_server;
pub mod reconnect;
pub mod session_manager;
pub mod udp;
pub mod message_queue;
pub mod partial_transfer;

// New builder API (recommended)
pub use builder::ClientBuilder;
pub use unified_client::{AsyncIgtlClient as UnifiedAsyncClient, SyncIgtlClient};

// Legacy API (deprecated)
pub use client::IgtlClient;
pub use server::{IgtlConnection, IgtlServer};
pub use async_client::{AsyncIgtlClient, AsyncIgtlReader, AsyncIgtlWriter};
pub use async_server::{
    AsyncIgtlConnection, AsyncIgtlConnectionReader, AsyncIgtlConnectionWriter, AsyncIgtlServer,
};
pub use tls_client::{insecure_tls_config, TlsIgtlClient};
pub use tls_server::{TlsIgtlConnection, TlsIgtlServer};
pub use reconnect::{ReconnectClient, ReconnectConfig};
pub use session_manager::{ClientId, ClientInfo, MessageHandler, SessionManager};
pub use udp::{UdpClient, UdpServer};
pub use message_queue::{MessageQueue, QueueConfig, QueueStats};
pub use partial_transfer::{
    PartialTransferManager, TransferConfig, TransferId, TransferInfo, TransferState,
};
