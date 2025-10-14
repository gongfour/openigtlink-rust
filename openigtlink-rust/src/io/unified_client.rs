//! Unified client types for OpenIGTLink
//!
//! This module provides simplified client enums that delegate to internal implementations.

use crate::error::Result;
use crate::io::sync_client::SyncTcpClient;
use crate::io::unified_async_client::UnifiedAsyncClient;
use crate::protocol::any_message::AnyMessage;
use crate::protocol::message::{IgtlMessage, Message};

/// Synchronous OpenIGTLink client
///
/// Simple wrapper around the synchronous TCP client.
///
/// **Recommended**: Use [`ClientBuilder`](crate::io::builder::ClientBuilder):
/// ```no_run
/// use openigtlink_rust::io::builder::ClientBuilder;
///
/// let client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;
/// # Ok::<(), openigtlink_rust::error::IgtlError>(())
/// ```
pub enum SyncIgtlClient {
    /// Standard TCP synchronous client
    TcpSync(SyncTcpClient),
}

impl SyncIgtlClient {
    /// Send a message to the server
    ///
    /// # Arguments
    /// * `msg` - Message to send
    ///
    /// # Returns
    /// Ok(()) on success, error otherwise
    #[inline(always)]
    pub fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        match self {
            SyncIgtlClient::TcpSync(client) => client.send(msg),
        }
    }

    /// Receive a message from the server
    ///
    /// # Returns
    /// Decoded message or error
    #[inline(always)]
    pub fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        match self {
            SyncIgtlClient::TcpSync(client) => client.receive(),
        }
    }

    /// Enable or disable CRC verification for received messages
    ///
    /// # Arguments
    /// * `verify` - true to enable CRC verification, false to disable
    #[inline(always)]
    pub fn set_verify_crc(&mut self, verify: bool) {
        match self {
            SyncIgtlClient::TcpSync(client) => client.set_verify_crc(verify),
        }
    }

    /// Set read timeout for socket operations
    ///
    /// # Arguments
    /// * `timeout` - Optional timeout duration. None for no timeout
    ///
    /// # Returns
    /// Ok(()) on success, error if the socket operation fails
    #[inline(always)]
    pub fn set_read_timeout(&self, timeout: Option<std::time::Duration>) -> Result<()> {
        match self {
            SyncIgtlClient::TcpSync(client) => client.set_read_timeout(timeout),
        }
    }

    /// Receive any message type dynamically without knowing the type in advance
    ///
    /// # Returns
    /// Decoded message wrapped in AnyMessage enum, or error
    #[inline(always)]
    pub fn receive_any(&mut self) -> Result<AnyMessage> {
        match self {
            SyncIgtlClient::TcpSync(client) => client.receive_any(),
        }
    }
}

/// Asynchronous OpenIGTLink client
///
/// Unified client that supports optional TLS and reconnection features
/// through internal state rather than separate variants.
///
/// **Recommended**: Use [`ClientBuilder`](crate::io::builder::ClientBuilder):
/// ```no_run
/// use openigtlink_rust::io::builder::ClientBuilder;
///
/// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// let client = ClientBuilder::new().tcp("127.0.0.1:18944").async_mode().build().await?;
/// # Ok(())
/// # }
/// ```
pub enum AsyncIgtlClient {
    /// Unified async client with optional TLS and reconnection
    Unified(UnifiedAsyncClient),
}

impl AsyncIgtlClient {
    /// Send a message to the server asynchronously
    ///
    /// # Arguments
    /// * `msg` - Message to send
    ///
    /// # Returns
    /// Ok(()) on success, error otherwise
    #[inline(always)]
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        match self {
            AsyncIgtlClient::Unified(client) => client.send(msg).await,
        }
    }

    /// Receive a message from the server asynchronously
    ///
    /// # Returns
    /// Decoded message or error
    #[inline(always)]
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        match self {
            AsyncIgtlClient::Unified(client) => client.receive().await,
        }
    }

    /// Enable or disable CRC verification for received messages
    ///
    /// # Arguments
    /// * `verify` - true to enable CRC verification, false to disable
    #[inline(always)]
    pub fn set_verify_crc(&mut self, verify: bool) {
        match self {
            AsyncIgtlClient::Unified(client) => client.set_verify_crc(verify),
        }
    }

    /// Get the number of reconnection attempts that have occurred
    ///
    /// # Returns
    /// The total count of successful reconnections
    #[inline(always)]
    pub fn reconnect_count(&self) -> usize {
        match self {
            AsyncIgtlClient::Unified(client) => client.reconnect_count(),
        }
    }

    /// Receive any message type dynamically without knowing the type in advance
    ///
    /// # Returns
    /// Decoded message wrapped in AnyMessage enum, or error
    #[inline(always)]
    pub async fn receive_any(&mut self) -> Result<AnyMessage> {
        match self {
            AsyncIgtlClient::Unified(client) => client.receive_any().await,
        }
    }
}
