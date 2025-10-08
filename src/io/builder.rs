//! Type-state builder pattern for OpenIGTLink clients
//!
//! This module provides a flexible, type-safe way to construct OpenIGTLink clients
//! with exactly the features you need. The builder uses Rust's type system to prevent
//! invalid configurations at compile time.
//!
//! # Design Philosophy
//!
//! Instead of creating separate types for every feature combination (which would lead
//! to exponential growth: TcpAsync, TcpAsyncTls, TcpAsyncReconnect, TcpAsyncTlsReconnect...),
//! this builder creates a single [`UnifiedAsyncClient`]
//! with optional features.
//!
//! **Benefits:**
//! - ✅ Compile-time safety: Invalid combinations caught at compile time
//! - ✅ No variant explosion: Scales to any number of features
//! - ✅ Zero runtime cost: `PhantomData` markers are optimized away
//! - ✅ Ergonomic API: Method chaining with clear intent
//!
//! # Type-State Pattern
//!
//! The builder uses type states to enforce valid construction:
//!
//! ```text
//! ClientBuilder<Unspecified, Unspecified>
//!   ├─ .tcp(addr)  → ClientBuilder<TcpConfigured, Unspecified>
//!   │   ├─ .sync()       → ClientBuilder<TcpConfigured, SyncMode>
//!   │   │   └─ .build()  → Result<SyncIgtlClient>
//!   │   └─ .async_mode() → ClientBuilder<TcpConfigured, AsyncMode>
//!   │       ├─ .with_tls(config)      → self
//!   │       ├─ .with_reconnect(cfg)   → self
//!   │       ├─ .verify_crc(bool)      → self
//!   │       └─ .build()               → Result<UnifiedAsyncClient>
//!   └─ .udp(addr)  → ClientBuilder<UdpConfigured, Unspecified>
//!       └─ .build() → Result<UdpClient>
//! ```
//!
//! Invalid state transitions result in **compile errors**, not runtime errors!
//!
//! # Examples
//!
//! ## Basic TCP Clients
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//!
//! # fn main() -> Result<(), openigtlink_rust::error::IgtlError> {
//! // Synchronous TCP client (blocking I/O)
//! let client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .sync()
//!     .build()?;
//! # Ok(())
//! # }
//!
//! // Asynchronous TCP client (Tokio)
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! # use openigtlink_rust::io::builder::ClientBuilder;
//! let client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .async_mode()
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## TLS-Encrypted Clients
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! // TLS client for secure hospital networks
//! let tls_config = rustls::ClientConfig::builder()
//!     .with_root_certificates(rustls::RootCertStore::empty())
//!     .with_no_client_auth();
//!
//! let client = ClientBuilder::new()
//!     .tcp("hospital-server.local:18944")
//!     .async_mode()
//!     .with_tls(Arc::new(tls_config))
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Auto-Reconnecting Clients
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//! use openigtlink_rust::io::reconnect::ReconnectConfig;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! // Client that auto-reconnects on network failures
//! let reconnect_config = ReconnectConfig::with_max_attempts(10);
//! let client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .async_mode()
//!     .with_reconnect(reconnect_config)
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Combined Features (TLS + Auto-Reconnect)
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//! use openigtlink_rust::io::reconnect::ReconnectConfig;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! // Production-ready client with encryption AND auto-reconnect
//! let tls_config = rustls::ClientConfig::builder()
//!     .with_root_certificates(rustls::RootCertStore::empty())
//!     .with_no_client_auth();
//!
//! let reconnect_config = ReconnectConfig::with_max_attempts(100);
//!
//! let client = ClientBuilder::new()
//!     .tcp("production-server:18944")
//!     .async_mode()
//!     .with_tls(Arc::new(tls_config))
//!     .with_reconnect(reconnect_config)
//!     .verify_crc(true)
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## UDP Client for Low-Latency Tracking
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//!
//! // UDP client for real-time surgical tool tracking (120+ Hz)
//! let client = ClientBuilder::new()
//!     .udp("127.0.0.1:18944")
//!     .build()?;
//! # Ok::<(), openigtlink_rust::error::IgtlError>(())
//! ```
//!
//! ## Compile-Time Error Prevention
//!
//! The following code will **not compile**:
//!
//! ```compile_fail
//! use openigtlink_rust::io::builder::ClientBuilder;
//!
//! // ERROR: This library does not implement UDP + TLS (DTLS)
//! let client = ClientBuilder::new()
//!     .udp("127.0.0.1:18944")
//!     .with_tls(config)  // ← Compile error: method not found
//!     .build()?;
//! ```
//!
//! **Note**: While DTLS (Datagram TLS) exists in theory, this library focuses on
//! TCP-based TLS as it's the standard for OpenIGTLink secure communications.

use crate::error::Result;
use crate::io::reconnect::ReconnectConfig;
use crate::io::sync_client::SyncTcpClient;
use crate::io::unified_async_client::UnifiedAsyncClient;
use crate::io::unified_client::{AsyncIgtlClient, SyncIgtlClient};
use crate::io::UdpClient;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio_rustls::rustls;

// ============================================================================
// State Marker Types
// ============================================================================

/// Unspecified protocol or mode state
///
/// This is the initial state before protocol or mode selection.
pub struct Unspecified;

/// TCP protocol configured state
///
/// Contains the server address for TCP connection.
#[allow(dead_code)]
pub struct TcpConfigured {
    pub(crate) addr: String,
}

/// UDP protocol configured state
///
/// Contains the server address for UDP communication.
/// Note: UDP only supports synchronous mode.
#[allow(dead_code)]
pub struct UdpConfigured {
    pub(crate) addr: String,
}

/// Synchronous (blocking) mode state
pub struct SyncMode;

/// Asynchronous (non-blocking) mode state
pub struct AsyncMode;

// ============================================================================
// ClientBuilder - Type-State Pattern
// ============================================================================

/// Type-state builder for OpenIGTLink clients
///
/// Uses compile-time type checking to ensure only valid client configurations
/// can be constructed.
///
/// # Type Parameters
/// * `Protocol` - Protocol state (Unspecified, TcpConfigured, UdpConfigured)
/// * `Mode` - Mode state (Unspecified, SyncMode, AsyncMode)
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::builder::ClientBuilder;
///
/// let client = ClientBuilder::new()
///     .tcp("127.0.0.1:18944")
///     .sync()
///     .build()?;
/// # Ok::<(), openigtlink_rust::error::IgtlError>(())
/// ```
pub struct ClientBuilder<Protocol = Unspecified, Mode = Unspecified> {
    protocol: Protocol,
    mode: PhantomData<Mode>,
    tls_config: Option<Arc<rustls::ClientConfig>>,
    reconnect_config: Option<ReconnectConfig>,
    verify_crc: bool,
}

// ============================================================================
// Initial Construction
// ============================================================================

impl ClientBuilder<Unspecified, Unspecified> {
    /// Create a new client builder
    ///
    /// This is the starting point for building any OpenIGTLink client.
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// let builder = ClientBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            protocol: Unspecified,
            mode: PhantomData,
            tls_config: None,
            reconnect_config: None,
            verify_crc: true,
        }
    }
}

impl Default for ClientBuilder<Unspecified, Unspecified> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Protocol Selection
// ============================================================================

impl ClientBuilder<Unspecified, Unspecified> {
    /// Select TCP protocol
    ///
    /// # Arguments
    /// * `addr` - Server address (e.g., "127.0.0.1:18944")
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// let builder = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944");
    /// ```
    pub fn tcp(self, addr: impl Into<String>) -> ClientBuilder<TcpConfigured, Unspecified> {
        ClientBuilder {
            protocol: TcpConfigured { addr: addr.into() },
            mode: PhantomData,
            tls_config: self.tls_config,
            reconnect_config: self.reconnect_config,
            verify_crc: self.verify_crc,
        }
    }

    /// Select UDP protocol
    ///
    /// Note: UDP automatically sets mode to SyncMode as UDP only supports
    /// synchronous operation.
    ///
    /// # Arguments
    /// * `addr` - Server address (e.g., "127.0.0.1:18944")
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// let builder = ClientBuilder::new()
    ///     .udp("127.0.0.1:18944");
    /// ```
    pub fn udp(self, addr: impl Into<String>) -> ClientBuilder<UdpConfigured, SyncMode> {
        ClientBuilder {
            protocol: UdpConfigured { addr: addr.into() },
            mode: PhantomData,
            tls_config: self.tls_config,
            reconnect_config: self.reconnect_config,
            verify_crc: self.verify_crc,
        }
    }
}

// ============================================================================
// Mode Selection (TCP only)
// ============================================================================

impl ClientBuilder<TcpConfigured, Unspecified> {
    /// Select synchronous (blocking) mode
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// let builder = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .sync();
    /// ```
    pub fn sync(self) -> ClientBuilder<TcpConfigured, SyncMode> {
        ClientBuilder {
            protocol: self.protocol,
            mode: PhantomData,
            tls_config: self.tls_config,
            reconnect_config: self.reconnect_config,
            verify_crc: self.verify_crc,
        }
    }

    /// Select asynchronous (non-blocking) mode
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// let builder = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .async_mode();
    /// ```
    pub fn async_mode(self) -> ClientBuilder<TcpConfigured, AsyncMode> {
        ClientBuilder {
            protocol: self.protocol,
            mode: PhantomData,
            tls_config: self.tls_config,
            reconnect_config: self.reconnect_config,
            verify_crc: self.verify_crc,
        }
    }
}

// ============================================================================
// TCP Sync Mode Configuration and Build
// ============================================================================

impl ClientBuilder<TcpConfigured, SyncMode> {
    /// Build a synchronous TCP client
    ///
    /// # Errors
    ///
    /// Returns error if connection fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// let client = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .sync()
    ///     .build()?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn build(self) -> Result<SyncIgtlClient> {
        let mut client = SyncTcpClient::connect(&self.protocol.addr)?;
        client.set_verify_crc(self.verify_crc);
        Ok(SyncIgtlClient::TcpSync(client))
    }
}

// ============================================================================
// TCP Async Mode Configuration and Build
// ============================================================================

impl ClientBuilder<TcpConfigured, AsyncMode> {
    /// Configure TLS encryption
    ///
    /// # Arguments
    /// * `config` - TLS client configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::builder::ClientBuilder;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
    /// let tls_config = rustls::ClientConfig::builder()
    ///     .with_root_certificates(rustls::RootCertStore::empty())
    ///     .with_no_client_auth();
    ///
    /// let client = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .async_mode()
    ///     .with_tls(Arc::new(tls_config))
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_tls(mut self, config: Arc<rustls::ClientConfig>) -> Self {
        self.tls_config = Some(config);
        self
    }

    /// Configure automatic reconnection
    ///
    /// # Arguments
    /// * `config` - Reconnection strategy configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::builder::ClientBuilder;
    /// use openigtlink_rust::io::reconnect::ReconnectConfig;
    ///
    /// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
    /// let client = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .async_mode()
    ///     .with_reconnect(ReconnectConfig::default())
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_reconnect(mut self, config: ReconnectConfig) -> Self {
        self.reconnect_config = Some(config);
        self
    }

    /// Build an asynchronous TCP client
    ///
    /// Creates the appropriate client variant based on configured options:
    /// - No options: Plain async TCP client
    /// - TLS only: TLS-encrypted async client
    /// - Reconnect only: Auto-reconnecting async client
    /// - TLS + Reconnect: TLS-encrypted auto-reconnecting client
    ///
    /// # Errors
    ///
    /// Returns error if connection fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
    /// // Plain async client
    /// let client = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .async_mode()
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(self) -> Result<AsyncIgtlClient> {
        let addr = self.protocol.addr;

        // Create base client (with or without TLS)
        let mut client = if let Some(tls_config) = self.tls_config {
            // TLS connection
            let (hostname, port) = parse_addr(&addr)?;
            UnifiedAsyncClient::connect_with_tls(&hostname, port, tls_config).await?
        } else {
            // Plain TCP connection
            UnifiedAsyncClient::connect(&addr).await?
        };

        // Add reconnection if configured
        if let Some(reconnect_config) = self.reconnect_config {
            client = client.with_reconnect(reconnect_config);
        }

        // Set CRC verification
        client.set_verify_crc(self.verify_crc);

        Ok(AsyncIgtlClient::Unified(client))
    }
}

// ============================================================================
// Common Configuration Methods
// ============================================================================

impl<Protocol, Mode> ClientBuilder<Protocol, Mode> {
    /// Enable or disable CRC verification for received messages
    ///
    /// Default: true (CRC verification enabled)
    ///
    /// # Arguments
    /// * `verify` - true to enable CRC verification, false to disable
    ///
    /// # Examples
    ///
    /// ```
    /// use openigtlink_rust::io::builder::ClientBuilder;
    ///
    /// let builder = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .sync()
    ///     .verify_crc(false);
    /// ```
    pub fn verify_crc(mut self, verify: bool) -> Self {
        self.verify_crc = verify;
        self
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse address string into hostname and port
///
/// # Arguments
/// * `addr` - Address string in format "hostname:port"
///
/// # Returns
/// Tuple of (hostname, port)
fn parse_addr(addr: &str) -> Result<(String, u16)> {
    let parts: Vec<&str> = addr.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(crate::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid address format: {}", addr),
        )));
    }

    let port = parts[0].parse::<u16>().map_err(|e| {
        crate::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid port number: {}", e),
        ))
    })?;

    let hostname = parts[1].to_string();

    // Validate hostname is not empty
    if hostname.is_empty() {
        return Err(crate::error::IgtlError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Hostname cannot be empty",
        )));
    }

    Ok((hostname, port))
}

// ============================================================================
// UDP Configuration and Build
// ============================================================================

impl ClientBuilder<UdpConfigured, SyncMode> {
    /// Build a UDP client
    ///
    /// UDP clients use a connectionless protocol and require specifying
    /// the target address for each send operation using `send_to()`.
    ///
    /// # Errors
    ///
    /// Returns error if binding to local address fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::builder::ClientBuilder;
    /// use openigtlink_rust::protocol::types::TransformMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let client = ClientBuilder::new()
    ///     .udp("0.0.0.0:0")
    ///     .build()?;
    ///
    /// let transform = TransformMessage::identity();
    /// let msg = IgtlMessage::new(transform, "Device")?;
    /// client.send_to(&msg, "127.0.0.1:18944")?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn build(self) -> Result<UdpClient> {
        UdpClient::bind(&self.protocol.addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_data_is_zero_size() {
        use std::mem::size_of;

        // PhantomData should have zero size
        assert_eq!(size_of::<PhantomData<SyncMode>>(), 0);
        assert_eq!(size_of::<PhantomData<AsyncMode>>(), 0);

        // Ensure the builder overhead is minimal
        let base_size = size_of::<ClientBuilder<Unspecified, Unspecified>>();

        // All builder variants should have similar size (protocol state adds addr String)
        // but Mode type parameter should add zero cost
        let tcp_unspecified = size_of::<ClientBuilder<TcpConfigured, Unspecified>>();
        let tcp_sync = size_of::<ClientBuilder<TcpConfigured, SyncMode>>();
        let tcp_async = size_of::<ClientBuilder<TcpConfigured, AsyncMode>>();

        // Mode change should not increase size
        assert_eq!(tcp_unspecified, tcp_sync);
        assert_eq!(tcp_unspecified, tcp_async);

        // Protocol state adds String, so should be larger than base
        assert!(tcp_unspecified > base_size);
    }

    #[test]
    fn test_builder_state_transitions() {
        // Initial state
        let builder = ClientBuilder::new();

        // TCP protocol selection
        let builder = builder.tcp("127.0.0.1:18944");

        // Mode selection
        let _sync_builder = builder.sync();

        // Restart for async test
        let builder = ClientBuilder::new().tcp("127.0.0.1:18944");
        let _async_builder = builder.async_mode();

        // UDP automatically sets sync mode
        let _udp_builder = ClientBuilder::new().udp("127.0.0.1:18944");
    }

    #[test]
    fn test_parse_addr() {
        // Valid addresses
        assert_eq!(
            parse_addr("localhost:18944").unwrap(),
            ("localhost".to_string(), 18944)
        );
        assert_eq!(
            parse_addr("127.0.0.1:8080").unwrap(),
            ("127.0.0.1".to_string(), 8080)
        );
        assert_eq!(
            parse_addr("example.com:443").unwrap(),
            ("example.com".to_string(), 443)
        );

        // Invalid addresses
        assert!(parse_addr("invalid").is_err());
        assert!(parse_addr("localhost:").is_err());
        assert!(parse_addr(":18944").is_err());
        assert!(parse_addr("localhost:abc").is_err());
    }

    #[test]
    fn test_builder_options() {
        // Verify CRC option
        let builder = ClientBuilder::new()
            .tcp("127.0.0.1:18944")
            .sync()
            .verify_crc(false);
        assert!(!builder.verify_crc);

        // TLS option
        let tls_config = Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(rustls::RootCertStore::empty())
                .with_no_client_auth(),
        );
        let builder = ClientBuilder::new()
            .tcp("127.0.0.1:18944")
            .async_mode()
            .with_tls(tls_config.clone());
        assert!(builder.tls_config.is_some());

        // Reconnect option
        let builder = ClientBuilder::new()
            .tcp("127.0.0.1:18944")
            .async_mode()
            .with_reconnect(ReconnectConfig::default());
        assert!(builder.reconnect_config.is_some());
    }
}
