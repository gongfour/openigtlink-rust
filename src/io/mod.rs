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

pub mod async_server;
pub mod builder;
mod common;
pub mod reconnect;
pub mod server;
mod sync_client;
pub mod tls_server;
pub mod udp;
pub mod unified_async_client;
pub mod unified_client;

// Client builder API (recommended)
pub use builder::ClientBuilder;
pub use reconnect::ReconnectConfig;
pub use unified_client::{AsyncIgtlClient, SyncIgtlClient};

// Server APIs
pub use async_server::{
    AsyncIgtlConnection, AsyncIgtlConnectionReader, AsyncIgtlConnectionWriter, AsyncIgtlServer,
};
pub use server::{IgtlConnection, IgtlServer};
pub use tls_server::{TlsIgtlConnection, TlsIgtlServer};

// UDP
pub use udp::{UdpClient, UdpServer};
