//! Network I/O module for OpenIGTLink communication
//!
//! Provides client and server implementations for OpenIGTLink connections
//! over both TCP (reliable) and UDP (low-latency) transports.
//!
//! # Client Creation
//!
//! Use [`ClientBuilder`] for type-safe client construction:
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! // TCP async client
//! let client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .async_mode()
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

mod common;
mod sync_client;
pub mod unified_async_client;
pub mod unified_client;
pub mod builder;
pub mod server;
pub mod async_server;
pub mod tls_server;
pub mod reconnect;
pub mod session_manager;
pub mod udp;
pub mod message_queue;
pub mod partial_transfer;

// Client builder API (recommended)
pub use builder::ClientBuilder;
pub use unified_client::{AsyncIgtlClient, SyncIgtlClient};
pub use reconnect::ReconnectConfig;

// Server APIs
pub use server::{IgtlConnection, IgtlServer};
pub use async_server::{
    AsyncIgtlConnection, AsyncIgtlConnectionReader, AsyncIgtlConnectionWriter, AsyncIgtlServer,
};
pub use tls_server::{TlsIgtlConnection, TlsIgtlServer};

// Session and queue management
pub use session_manager::{ClientId, ClientInfo, MessageHandler, SessionManager};
pub use message_queue::{MessageQueue, QueueConfig, QueueStats};
pub use partial_transfer::{
    PartialTransferManager, TransferConfig, TransferId, TransferInfo, TransferState,
};

// UDP
pub use udp::{UdpClient, UdpServer};
