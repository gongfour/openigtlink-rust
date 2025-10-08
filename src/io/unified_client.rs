//! Unified client types for OpenIGTLink
//!
//! This module provides simplified client types that avoid combinatorial explosion
//! of variants by using internal optional features.

use crate::error::Result;
use crate::io::IgtlClient;
use crate::io::unified_async_client::UnifiedAsyncClient;
use crate::protocol::message::{IgtlMessage, Message};

/// Synchronous OpenIGTLink client
///
/// Simple wrapper around the synchronous TCP client.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::unified_client::SyncIgtlClient;
/// use openigtlink_rust::io::IgtlClient;
///
/// let tcp_client = IgtlClient::connect("127.0.0.1:18944")?;
/// let client = SyncIgtlClient::TcpSync(tcp_client);
/// # Ok::<(), openigtlink_rust::error::IgtlError>(())
/// ```
pub enum SyncIgtlClient {
    /// Standard TCP synchronous client
    TcpSync(IgtlClient),
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
}

/// Asynchronous OpenIGTLink client
///
/// Unified client that supports optional TLS and reconnection features
/// through internal state rather than separate variants.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::unified_client::AsyncIgtlClient;
/// use openigtlink_rust::io::unified_async_client::UnifiedAsyncClient;
///
/// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// let client = UnifiedAsyncClient::connect("127.0.0.1:18944").await?;
/// let mut unified = AsyncIgtlClient::Unified(client);
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
}
