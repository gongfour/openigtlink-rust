//! Unified client enums for type-safe protocol and mode selection
//!
//! This module provides unified enum types that wrap different client implementations,
//! enabling compile-time type-safe selection of protocol (TCP/UDP) and mode (Sync/Async).

use crate::error::Result;
use crate::io::{IgtlClient, TlsIgtlClient, ReconnectClient};
use crate::io::tls_reconnect::TcpAsyncTlsReconnectClient;
use crate::protocol::message::{IgtlMessage, Message};

/// Synchronous OpenIGTLink client variants
///
/// This enum provides a unified interface for all synchronous (blocking) client types.
/// Currently supports TCP protocol only. UDP clients use a different API pattern.
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

/// Asynchronous OpenIGTLink client variants
///
/// This enum provides a unified interface for all asynchronous (non-blocking) client types.
/// Supports TCP with various feature combinations: plain, TLS, reconnect, and TLS+reconnect.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::unified_client::AsyncIgtlClient;
/// use openigtlink_rust::io::AsyncIgtlClient as TcpAsyncClient;
///
/// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// let tcp_client = TcpAsyncClient::connect("127.0.0.1:18944").await?;
/// let client = AsyncIgtlClient::TcpAsync(tcp_client);
/// # Ok(())
/// # }
/// ```
pub enum AsyncIgtlClient {
    /// Plain TCP asynchronous client
    TcpAsync(crate::io::AsyncIgtlClient),
    /// TCP asynchronous client with TLS encryption
    TcpAsyncTls(TlsIgtlClient),
    /// TCP asynchronous client with automatic reconnection
    TcpAsyncReconnect(ReconnectClient),
    /// TCP asynchronous client with both TLS and automatic reconnection
    TcpAsyncTlsReconnect(TcpAsyncTlsReconnectClient),
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
            AsyncIgtlClient::TcpAsync(client) => client.send(msg).await,
            AsyncIgtlClient::TcpAsyncTls(client) => client.send(msg).await,
            AsyncIgtlClient::TcpAsyncReconnect(client) => client.send(msg).await,
            AsyncIgtlClient::TcpAsyncTlsReconnect(client) => client.send(msg).await,
        }
    }

    /// Receive a message from the server asynchronously
    ///
    /// # Returns
    /// Decoded message or error
    #[inline(always)]
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        match self {
            AsyncIgtlClient::TcpAsync(client) => client.receive().await,
            AsyncIgtlClient::TcpAsyncTls(client) => client.receive().await,
            AsyncIgtlClient::TcpAsyncReconnect(client) => client.receive().await,
            AsyncIgtlClient::TcpAsyncTlsReconnect(client) => client.receive().await,
        }
    }

    /// Enable or disable CRC verification for received messages
    ///
    /// # Arguments
    /// * `verify` - true to enable CRC verification, false to disable
    #[inline(always)]
    pub fn set_verify_crc(&mut self, verify: bool) {
        match self {
            AsyncIgtlClient::TcpAsync(client) => client.set_verify_crc(verify),
            AsyncIgtlClient::TcpAsyncTls(client) => client.set_verify_crc(verify),
            AsyncIgtlClient::TcpAsyncReconnect(client) => client.set_verify_crc(verify),
            AsyncIgtlClient::TcpAsyncTlsReconnect(client) => client.set_verify_crc(verify),
        }
    }
}

