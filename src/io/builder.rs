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

use crate::io::reconnect::ReconnectConfig;
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
}
