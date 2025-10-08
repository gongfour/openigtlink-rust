//! TLS-encrypted client with automatic reconnection
//!
//! Combines TLS encryption with automatic reconnection for secure,
//! resilient communication in unstable network environments.

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

/// TLS-encrypted client with automatic reconnection support
///
/// Provides both TLS encryption and automatic reconnection with exponential backoff.
/// Ideal for secure communication over unreliable networks.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::tls_reconnect::TcpAsyncTlsReconnectClient;
/// use openigtlink_rust::io::reconnect::ReconnectConfig;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// let tls_config = rustls::ClientConfig::builder()
///     .with_root_certificates(rustls::RootCertStore::empty())
///     .with_no_client_auth();
///
/// let reconnect_config = ReconnectConfig::default();
///
/// let client = TcpAsyncTlsReconnectClient::connect(
///     "secure-server.local",
///     18944,
///     Arc::new(tls_config),
///     reconnect_config
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub struct TcpAsyncTlsReconnectClient {
    hostname: String,
    port: u16,
    tls_config: Arc<rustls::ClientConfig>,
    reconnect_config: ReconnectConfig,
    stream: Option<TlsStream<TcpStream>>,
    reconnect_count: usize,
    verify_crc: bool,
}

impl TcpAsyncTlsReconnectClient {
    /// Connect to a TLS-enabled server with automatic reconnection
    ///
    /// # Arguments
    ///
    /// * `hostname` - Server hostname (for SNI and certificate validation)
    /// * `port` - Server port
    /// * `tls_config` - TLS client configuration
    /// * `reconnect_config` - Reconnection strategy configuration
    ///
    /// # Errors
    ///
    /// Returns error if initial connection fails and max attempts is reached
    pub async fn connect(
        hostname: &str,
        port: u16,
        tls_config: Arc<rustls::ClientConfig>,
        reconnect_config: ReconnectConfig,
    ) -> Result<Self> {
        info!(
            hostname = hostname,
            port = port,
            "Creating TLS reconnecting client"
        );

        let mut client = Self {
            hostname: hostname.to_string(),
            port,
            tls_config,
            reconnect_config,
            stream: None,
            reconnect_count: 0,
            verify_crc: true,
        };

        // Initial connection attempt
        client.ensure_connected().await?;

        Ok(client)
    }

    /// Attempt to establish a TLS connection
    async fn try_connect(
        hostname: &str,
        port: u16,
        tls_config: Arc<rustls::ClientConfig>,
    ) -> Result<TlsStream<TcpStream>> {
        let addr = format!("{}:{}", hostname, port);

        debug!(addr = %addr, "Attempting TLS connection");

        // Connect TCP
        let tcp_stream = TcpStream::connect(&addr).await?;
        let local_addr = tcp_stream.local_addr()?;

        // Perform TLS handshake
        let server_name = ServerName::try_from(hostname.to_string()).map_err(|e| {
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid hostname: {}", e),
            ))
        })?;

        let connector = TlsConnector::from(tls_config);
        let stream = connector.connect(server_name, tcp_stream).await.map_err(|e| {
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

        Ok(stream)
    }

    /// Ensure we have a valid TLS connection, reconnecting if necessary
    async fn ensure_connected(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let mut attempt = 0;

        loop {
            if let Some(max) = self.reconnect_config.max_attempts {
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

            let delay = self.reconnect_config.delay_for_attempt(attempt);
            if attempt > 0 {
                info!(
                    attempt = attempt + 1,
                    delay_ms = delay.as_millis(),
                    "Reconnecting with TLS..."
                );
                sleep(delay).await;
            }

            match Self::try_connect(&self.hostname, self.port, self.tls_config.clone()).await {
                Ok(stream) => {
                    self.stream = Some(stream);
                    if attempt > 0 {
                        self.reconnect_count += 1;
                        info!(
                            reconnect_count = self.reconnect_count,
                            "TLS reconnection successful"
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        attempt = attempt + 1,
                        error = %e,
                        "TLS reconnection attempt failed"
                    );
                    attempt += 1;
                }
            }
        }
    }

    /// Get the number of times this client has reconnected
    pub fn reconnect_count(&self) -> usize {
        self.reconnect_count
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Enable or disable CRC verification
    pub fn set_verify_crc(&mut self, verify: bool) {
        self.verify_crc = verify;
    }

    /// Get current CRC verification setting
    pub fn verify_crc(&self) -> bool {
        self.verify_crc
    }

    /// Send a message over TLS, reconnecting if necessary
    ///
    /// If the send fails due to connection issues, this will automatically
    /// attempt to reconnect and retry the send.
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            size = data.len(),
            "Sending message over TLS (with auto-reconnect)"
        );

        loop {
            self.ensure_connected().await?;

            if let Some(stream) = &mut self.stream {
                match stream.write_all(&data).await {
                    Ok(_) => {
                        stream.flush().await?;
                        trace!(
                            msg_type = msg_type,
                            bytes_sent = data.len(),
                            "Message sent successfully over TLS"
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        warn!(error = %e, "TLS send failed, will reconnect");
                        self.stream = None;
                        // Loop will retry after reconnection
                    }
                }
            }
        }
    }

    /// Receive a message over TLS, reconnecting if necessary
    ///
    /// If the receive fails due to connection issues, this will automatically
    /// attempt to reconnect.
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        loop {
            self.ensure_connected().await?;

            if let Some(stream) = &mut self.stream {
                // Read header
                let mut header_buf = vec![0u8; Header::SIZE];
                match stream.read_exact(&mut header_buf).await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!(error = %e, "TLS header read failed, will reconnect");
                        self.stream = None;
                        continue;
                    }
                }

                let header = match Header::decode(&header_buf) {
                    Ok(h) => h,
                    Err(e) => {
                        warn!(error = %e, "Header decode failed");
                        return Err(e);
                    }
                };

                let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
                let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

                debug!(
                    msg_type = msg_type,
                    device_name = device_name,
                    body_size = header.body_size,
                    version = header.version,
                    "Received message header over TLS"
                );

                // Read body
                let mut body_buf = vec![0u8; header.body_size as usize];
                match stream.read_exact(&mut body_buf).await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!(error = %e, "TLS body read failed, will reconnect");
                        self.stream = None;
                        continue;
                    }
                }

                trace!(
                    msg_type = msg_type,
                    bytes_read = body_buf.len(),
                    "Message body received over TLS"
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
            }
        }
    }
}
