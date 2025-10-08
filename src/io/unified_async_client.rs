//! Unified async client with optional TLS and reconnection
//!
//! This module provides `UnifiedAsyncClient`, a single async client type that elegantly
//! handles all feature combinations (TLS, reconnection) through internal state management.
//!
//! # Design Philosophy
//!
//! Traditional approach would create separate types for each feature combination:
//! - `TcpAsync`, `TcpAsyncTls`, `TcpAsyncReconnect`, `TcpAsyncTlsReconnect`...
//! - This leads to **variant explosion**: 2 features = 4 types, 3 features = 8 types, etc.
//!
//! **Our approach**: Single `UnifiedAsyncClient` with optional features:
//! - Internal `Transport` enum: `Plain(TcpStream)` or `Tls(TlsStream)`
//! - Optional `reconnect_config: Option<ReconnectConfig>`
//! - ✅ Scales linearly with features (not exponentially!)
//! - ✅ Easy to add new features (compression, authentication, etc.)
//! - ✅ Maintains type safety through builder pattern
//!
//! # Architecture
//!
//! ```text
//! UnifiedAsyncClient
//! ├─ transport: Option<Transport>
//! │  ├─ Plain(TcpStream)     ← Regular TCP
//! │  └─ Tls(TlsStream)       ← TLS-encrypted TCP
//! ├─ reconnect_config: Option<ReconnectConfig>
//! │  ├─ None                 ← No auto-reconnection
//! │  └─ Some(config)         ← Auto-reconnect with backoff
//! ├─ conn_params: ConnectionParams (host, port, TLS config)
//! └─ verify_crc: bool        ← CRC verification
//! ```
//!
//! # Examples
//!
//! ## Plain TCP Connection
//!
//! ```no_run
//! use openigtlink_rust::io::unified_async_client::UnifiedAsyncClient;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! let client = UnifiedAsyncClient::connect("127.0.0.1:18944").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## TLS-Encrypted Connection
//!
//! ```no_run
//! use openigtlink_rust::io::unified_async_client::UnifiedAsyncClient;
//! use openigtlink_rust::io::tls_client::insecure_tls_config;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! let tls_config = Arc::new(insecure_tls_config());
//! let client = UnifiedAsyncClient::connect_with_tls(
//!     "hospital-server.local",
//!     18944,
//!     tls_config
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## With Auto-Reconnection
//!
//! ```no_run
//! use openigtlink_rust::io::unified_async_client::UnifiedAsyncClient;
//! use openigtlink_rust::io::reconnect::ReconnectConfig;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! let mut client = UnifiedAsyncClient::connect("127.0.0.1:18944").await?;
//!
//! // Enable auto-reconnection
//! let reconnect_config = ReconnectConfig::with_max_attempts(10);
//! client = client.with_reconnect(reconnect_config);
//! # Ok(())
//! # }
//! ```
//!
//! ## TLS + Auto-Reconnect (Previously Impossible!)
//!
//! ```no_run
//! use openigtlink_rust::io::unified_async_client::UnifiedAsyncClient;
//! use openigtlink_rust::io::tls_client::insecure_tls_config;
//! use openigtlink_rust::io::reconnect::ReconnectConfig;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! let tls_config = Arc::new(insecure_tls_config());
//! let mut client = UnifiedAsyncClient::connect_with_tls(
//!     "production-server",
//!     18944,
//!     tls_config
//! ).await?;
//!
//! // Add auto-reconnection to TLS client
//! let reconnect_config = ReconnectConfig::with_max_attempts(100);
//! client = client.with_reconnect(reconnect_config);
//! # Ok(())
//! # }
//! ```
//!
//! # Prefer Using the Builder
//!
//! While you can create `UnifiedAsyncClient` directly, it's recommended to use
//! [`ClientBuilder`](crate::io::builder::ClientBuilder) for better ergonomics and type safety:
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//! use openigtlink_rust::io::tls_client::insecure_tls_config;
//! use openigtlink_rust::io::reconnect::ReconnectConfig;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! let client = ClientBuilder::new()
//!     .tcp("production-server:18944")
//!     .async_mode()
//!     .with_tls(Arc::new(insecure_tls_config()))
//!     .with_reconnect(ReconnectConfig::with_max_attempts(100))
//!     .verify_crc(true)
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use crate::error::{IgtlError, Result};
use crate::io::reconnect::ReconnectConfig;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use rustls::pki_types::ServerName;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_rustls::client::TlsStream;
use tokio_rustls::{rustls, TlsConnector};
use tracing::{debug, info, trace, warn};

/// Transport type for the async client
enum Transport {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl Transport {
    async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        match self {
            Transport::Plain(stream) => {
                stream.write_all(data).await?;
                Ok(())
            }
            Transport::Tls(stream) => {
                stream.write_all(data).await?;
                Ok(())
            }
        }
    }

    async fn flush(&mut self) -> Result<()> {
        match self {
            Transport::Plain(stream) => {
                stream.flush().await?;
                Ok(())
            }
            Transport::Tls(stream) => {
                stream.flush().await?;
                Ok(())
            }
        }
    }

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        match self {
            Transport::Plain(stream) => {
                stream.read_exact(buf).await?;
                Ok(())
            }
            Transport::Tls(stream) => {
                stream.read_exact(buf).await?;
                Ok(())
            }
        }
    }
}

/// Connection parameters for reconnection
struct ConnectionParams {
    addr: String,
    hostname: Option<String>,
    port: Option<u16>,
    tls_config: Option<Arc<rustls::ClientConfig>>,
}

/// Unified async OpenIGTLink client
///
/// Supports optional TLS encryption and automatic reconnection without
/// combinatorial type explosion.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::unified_async_client::UnifiedAsyncClient;
///
/// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// // Plain TCP client
/// let client = UnifiedAsyncClient::connect("127.0.0.1:18944").await?;
///
/// // With TLS
/// let tls_config = rustls::ClientConfig::builder()
///     .with_root_certificates(rustls::RootCertStore::empty())
///     .with_no_client_auth();
/// let client = UnifiedAsyncClient::connect_with_tls(
///     "localhost",
///     18944,
///     std::sync::Arc::new(tls_config)
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub struct UnifiedAsyncClient {
    transport: Option<Transport>,
    conn_params: ConnectionParams,
    reconnect_config: Option<ReconnectConfig>,
    reconnect_count: usize,
    verify_crc: bool,
}

impl UnifiedAsyncClient {
    /// Connect to a plain TCP server
    ///
    /// # Arguments
    /// * `addr` - Server address (e.g., "127.0.0.1:18944")
    pub async fn connect(addr: &str) -> Result<Self> {
        info!(addr = addr, "Connecting to OpenIGTLink server");
        let stream = TcpStream::connect(addr).await?;
        let local_addr = stream.local_addr()?;
        info!(
            local_addr = %local_addr,
            remote_addr = addr,
            "Connected to OpenIGTLink server"
        );

        Ok(Self {
            transport: Some(Transport::Plain(stream)),
            conn_params: ConnectionParams {
                addr: addr.to_string(),
                hostname: None,
                port: None,
                tls_config: None,
            },
            reconnect_config: None,
            reconnect_count: 0,
            verify_crc: true,
        })
    }

    /// Connect to a TLS-enabled server
    ///
    /// # Arguments
    /// * `hostname` - Server hostname (for SNI)
    /// * `port` - Server port
    /// * `tls_config` - TLS client configuration
    pub async fn connect_with_tls(
        hostname: &str,
        port: u16,
        tls_config: Arc<rustls::ClientConfig>,
    ) -> Result<Self> {
        info!(
            hostname = hostname,
            port = port,
            "Connecting to TLS-enabled OpenIGTLink server"
        );

        let addr = format!("{}:{}", hostname, port);
        let tcp_stream = TcpStream::connect(&addr).await?;
        let local_addr = tcp_stream.local_addr()?;

        let server_name = ServerName::try_from(hostname.to_string()).map_err(|e| {
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid hostname: {}", e),
            ))
        })?;

        let connector = TlsConnector::from(tls_config.clone());
        let tls_stream = connector.connect(server_name, tcp_stream).await.map_err(|e| {
            warn!(error = %e, "TLS handshake failed");
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("TLS handshake failed: {}", e),
            ))
        })?;

        info!(
            local_addr = %local_addr,
            remote_addr = %addr,
            "TLS connection established"
        );

        Ok(Self {
            transport: Some(Transport::Tls(tls_stream)),
            conn_params: ConnectionParams {
                addr,
                hostname: Some(hostname.to_string()),
                port: Some(port),
                tls_config: Some(tls_config),
            },
            reconnect_config: None,
            reconnect_count: 0,
            verify_crc: true,
        })
    }

    /// Enable automatic reconnection
    ///
    /// # Arguments
    /// * `config` - Reconnection configuration
    pub fn with_reconnect(mut self, config: ReconnectConfig) -> Self {
        self.reconnect_config = Some(config);
        self
    }

    /// Enable or disable CRC verification
    pub fn set_verify_crc(&mut self, verify: bool) {
        self.verify_crc = verify;
    }

    /// Get current CRC verification setting
    pub fn verify_crc(&self) -> bool {
        self.verify_crc
    }

    /// Get reconnection count
    pub fn reconnect_count(&self) -> usize {
        self.reconnect_count
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.transport.is_some()
    }

    /// Ensure we have a valid connection, reconnecting if necessary
    async fn ensure_connected(&mut self) -> Result<()> {
        if self.transport.is_some() {
            return Ok(());
        }

        let Some(ref config) = self.reconnect_config else {
            return Err(IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "Connection lost and reconnection is not enabled",
            )));
        };

        let mut attempt = 0;

        loop {
            if let Some(max) = config.max_attempts {
                if attempt >= max {
                    warn!(
                        attempts = attempt,
                        max_attempts = max,
                        "Max reconnection attempts reached"
                    );
                    return Err(IgtlError::Io(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "Max reconnection attempts exceeded",
                    )));
                }
            }

            let delay = config.delay_for_attempt(attempt);
            if attempt > 0 {
                info!(
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis(),
                    "Reconnecting..."
                );
                sleep(delay).await;
            }

            let result = if let Some(ref tls_config) = self.conn_params.tls_config {
                // TLS reconnection
                let hostname = self.conn_params.hostname.as_ref().unwrap();
                let port = self.conn_params.port.unwrap();
                Self::connect_with_tls(hostname, port, tls_config.clone()).await
            } else {
                // Plain TCP reconnection
                Self::connect(&self.conn_params.addr).await
            };

            match result {
                Ok(new_client) => {
                    self.transport = new_client.transport;
                    if attempt > 0 {
                        self.reconnect_count += 1;
                        info!(
                            reconnect_count = self.reconnect_count,
                            "Reconnection successful"
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "Reconnection attempt failed"
                    );
                    attempt += 1;
                }
            }
        }
    }

    /// Send a message
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            size = data.len(),
            "Sending message"
        );

        loop {
            if self.reconnect_config.is_some() {
                self.ensure_connected().await?;
            }

            if let Some(transport) = &mut self.transport {
                match transport.write_all(&data).await {
                    Ok(_) => {
                        transport.flush().await?;
                        trace!(
                            msg_type = msg_type,
                            bytes_sent = data.len(),
                            "Message sent successfully"
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        if self.reconnect_config.is_some() {
                            warn!(error = %e, "Send failed, will reconnect");
                            self.transport = None;
                            // Loop will retry after reconnection
                        } else {
                            return Err(e);
                        }
                    }
                }
            } else {
                return Err(IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Not connected",
                )));
            }
        }
    }

    /// Receive a message
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        loop {
            if self.reconnect_config.is_some() {
                self.ensure_connected().await?;
            }

            if let Some(transport) = &mut self.transport {
                // Read header
                let mut header_buf = vec![0u8; Header::SIZE];
                match transport.read_exact(&mut header_buf).await {
                    Ok(_) => {}
                    Err(e) => {
                        if self.reconnect_config.is_some() {
                            warn!(error = %e, "Header read failed, will reconnect");
                            self.transport = None;
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                }

                let header = Header::decode(&header_buf)?;
                let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
                let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

                debug!(
                    msg_type = msg_type,
                    device_name = device_name,
                    body_size = header.body_size,
                    version = header.version,
                    "Received message header"
                );

                // Read body
                let mut body_buf = vec![0u8; header.body_size as usize];
                match transport.read_exact(&mut body_buf).await {
                    Ok(_) => {}
                    Err(e) => {
                        if self.reconnect_config.is_some() {
                            warn!(error = %e, "Body read failed, will reconnect");
                            self.transport = None;
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                }

                trace!(
                    msg_type = msg_type,
                    bytes_read = body_buf.len(),
                    "Message body received"
                );

                // Decode full message
                let mut full_msg = header_buf;
                full_msg.extend_from_slice(&body_buf);

                let result = IgtlMessage::decode_with_options(&full_msg, self.verify_crc);

                match &result {
                    Ok(_) => {
                        debug!(
                            msg_type = msg_type,
                            device_name = device_name,
                            "Message decoded successfully"
                        );
                    }
                    Err(e) => {
                        warn!(
                            msg_type = msg_type,
                            error = %e,
                            "Failed to decode message"
                        );
                    }
                }

                return result;
            } else {
                return Err(IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotConnected,
                    "Not connected",
                )));
            }
        }
    }
}
