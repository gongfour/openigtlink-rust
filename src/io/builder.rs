//! Type-state builder pattern for OpenIGTLink clients
//!
//! Provides compile-time type-safe construction of clients with various protocol
//! and mode combinations. Invalid combinations (e.g., UDP with TLS) are prevented
//! at compile time.
//!
//! # Type-State Pattern
//!
//! This builder uses the type-state pattern to ensure correctness:
//! - Protocol state: Unspecified -> TcpConfigured or UdpConfigured
//! - Mode state: Unspecified -> SyncMode or AsyncMode
//!
//! Invalid state transitions result in compile errors.
//!
//! # Examples
//!
//! ```no_run
//! use openigtlink_rust::io::builder::ClientBuilder;
//!
//! // TCP Sync client
//! let client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .sync()
//!     .build()?;
//!
//! // TCP Async client with TLS
//! # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
//! let client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .async_mode()
//!     .with_tls()
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::io::reconnect::ReconnectConfig;
use crate::io::unified_client::{AsyncIgtlClient, SyncIgtlClient};
use crate::io::{AsyncIgtlClient as TcpAsyncClient, IgtlClient, ReconnectClient, TlsIgtlClient};
use crate::io::tls_reconnect::TcpAsyncTlsReconnectClient;
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
        let mut client = IgtlClient::connect(&self.protocol.addr)?;
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

        let client = match (self.tls_config, self.reconnect_config) {
            // Plain async TCP
            (None, None) => {
                let mut client = TcpAsyncClient::connect(&addr).await?;
                client.set_verify_crc(self.verify_crc);
                AsyncIgtlClient::TcpAsync(client)
            }

            // TLS only
            (Some(tls_config), None) => {
                // Parse hostname and port from addr
                let (hostname, port) = parse_addr(&addr)?;
                let mut client =
                    TlsIgtlClient::connect_with_config(&hostname, port, (*tls_config).clone())
                        .await?;
                client.set_verify_crc(self.verify_crc);
                AsyncIgtlClient::TcpAsyncTls(client)
            }

            // Reconnect only
            (None, Some(reconnect_config)) => {
                let mut client = ReconnectClient::connect(&addr, reconnect_config).await?;
                client.set_verify_crc(self.verify_crc);
                AsyncIgtlClient::TcpAsyncReconnect(client)
            }

            // TLS + Reconnect
            (Some(tls_config), Some(reconnect_config)) => {
                let (hostname, port) = parse_addr(&addr)?;
                let mut client = TcpAsyncTlsReconnectClient::connect(
                    &hostname,
                    port,
                    tls_config,
                    reconnect_config,
                )
                .await?;
                client.set_verify_crc(self.verify_crc);
                AsyncIgtlClient::TcpAsyncTlsReconnect(client)
            }
        };

        Ok(client)
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
