//! Automatic reconnection with exponential backoff
//!
//! Provides resilient client connections that automatically reconnect
//! after network failures.

use crate::error::{IgtlError, Result};
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Reconnection strategy configuration
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts (None = infinite)
    pub max_attempts: Option<usize>,
    /// Initial delay before first reconnection attempt
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
    /// Backoff multiplier (delay is multiplied by this after each attempt)
    pub backoff_multiplier: f64,
    /// Whether to add random jitter to delays
    pub use_jitter: bool,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: Some(10),
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl ReconnectConfig {
    /// Create config with infinite retries
    pub fn infinite() -> Self {
        Self {
            max_attempts: None,
            ..Default::default()
        }
    }

    /// Create config with specific max attempts
    pub fn with_max_attempts(attempts: usize) -> Self {
        Self {
            max_attempts: Some(attempts),
            ..Default::default()
        }
    }

    /// Create config with custom delays
    pub fn with_delays(initial: Duration, max: Duration) -> Self {
        Self {
            initial_delay: initial,
            max_delay: max,
            ..Default::default()
        }
    }

    /// Calculate delay for a given attempt number
    fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let delay_ms = self.initial_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);

        let mut delay = Duration::from_millis(delay_ms.min(self.max_delay.as_millis() as f64) as u64);

        // Add jitter if enabled (0-25% random variation)
        if self.use_jitter {
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hash, Hasher};

            let mut hasher = RandomState::new().build_hasher();
            attempt.hash(&mut hasher);
            let hash = hasher.finish();
            let jitter = (hash % 25) as f64 / 100.0; // 0-25%

            let jitter_ms = (delay.as_millis() as f64 * jitter) as u64;
            delay = Duration::from_millis(delay.as_millis() as u64 + jitter_ms);
        }

        delay
    }
}

/// Auto-reconnecting OpenIGTLink client
///
/// Automatically reconnects after connection failures with exponential backoff.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::{ReconnectClient, ReconnectConfig};
/// use openigtlink_rust::protocol::types::StatusMessage;
/// use openigtlink_rust::protocol::message::IgtlMessage;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = ReconnectConfig::default();
///     let mut client = ReconnectClient::connect("127.0.0.1:18944", config).await?;
///
///     // Will automatically reconnect if connection drops
///     let status = StatusMessage::ok("Hello");
///     let msg = IgtlMessage::new(status, "ReconnectClient")?;
///     client.send(&msg).await?;
///
///     Ok(())
/// }
/// ```
pub struct ReconnectClient {
    addr: String,
    stream: Option<TcpStream>,
    config: ReconnectConfig,
    verify_crc: bool,
    reconnect_count: usize,
}

impl ReconnectClient {
    /// Create a new reconnecting client
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "127.0.0.1:18944")
    /// * `config` - Reconnection configuration
    pub async fn connect(addr: &str, config: ReconnectConfig) -> Result<Self> {
        info!(addr = addr, "Creating reconnecting client");

        let stream = Self::try_connect(addr).await?;

        Ok(ReconnectClient {
            addr: addr.to_string(),
            stream: Some(stream),
            config,
            verify_crc: true,
            reconnect_count: 0,
        })
    }

    /// Attempt to connect to the server
    async fn try_connect(addr: &str) -> Result<TcpStream> {
        debug!(addr = addr, "Attempting connection");
        let stream = TcpStream::connect(addr).await?;
        info!(addr = addr, "Connected successfully");
        Ok(stream)
    }

    /// Ensure we have a valid connection, reconnecting if necessary
    async fn ensure_connected(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let mut attempt = 0;

        loop {
            if let Some(max) = self.config.max_attempts {
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

            let delay = self.config.delay_for_attempt(attempt);
            info!(
                attempt = attempt + 1,
                delay_ms = delay.as_millis(),
                "Reconnecting..."
            );

            sleep(delay).await;

            match Self::try_connect(&self.addr).await {
                Ok(stream) => {
                    self.stream = Some(stream);
                    self.reconnect_count += 1;
                    info!(
                        reconnect_count = self.reconnect_count,
                        "Reconnection successful"
                    );
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

    /// Send a message, reconnecting if necessary
    ///
    /// If the send fails due to connection issues, this will automatically
    /// attempt to reconnect and retry the send.
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            size = data.len(),
            "Sending message (with auto-reconnect)"
        );

        loop {
            self.ensure_connected().await?;

            if let Some(stream) = &mut self.stream {
                match stream.write_all(&data).await {
                    Ok(_) => {
                        stream.flush().await?;
                        debug!(msg_type = msg_type, "Message sent successfully");
                        return Ok(());
                    }
                    Err(e) => {
                        warn!(error = %e, "Send failed, will reconnect");
                        self.stream = None;
                        // Loop will retry after reconnection
                    }
                }
            }
        }
    }

    /// Receive a message, reconnecting if necessary
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
                        warn!(error = %e, "Header read failed, will reconnect");
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
                debug!(
                    msg_type = msg_type,
                    body_size = header.body_size,
                    "Received message header"
                );

                // Read body
                let mut body_buf = vec![0u8; header.body_size as usize];
                match stream.read_exact(&mut body_buf).await {
                    Ok(_) => {}
                    Err(e) => {
                        warn!(error = %e, "Body read failed, will reconnect");
                        self.stream = None;
                        continue;
                    }
                }

                let mut full_msg = header_buf;
                full_msg.extend_from_slice(&body_buf);

                return IgtlMessage::decode_with_options(&full_msg, self.verify_crc);
            }
        }
    }

    /// Manually trigger reconnection
    ///
    /// Useful for testing or forcing a reconnection.
    pub async fn reconnect(&mut self) -> Result<()> {
        info!("Manual reconnection triggered");
        self.stream = None;
        self.ensure_connected().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnect_config_defaults() {
        let config = ReconnectConfig::default();
        assert_eq!(config.max_attempts, Some(10));
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.use_jitter, true);
    }

    #[test]
    fn test_reconnect_config_infinite() {
        let config = ReconnectConfig::infinite();
        assert_eq!(config.max_attempts, None);
    }

    #[test]
    fn test_reconnect_config_delay_calculation() {
        let config = ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            use_jitter: false,
            ..Default::default()
        };

        // Attempt 0: 100ms
        let delay0 = config.delay_for_attempt(0);
        assert_eq!(delay0, Duration::from_millis(100));

        // Attempt 1: 200ms
        let delay1 = config.delay_for_attempt(1);
        assert_eq!(delay1, Duration::from_millis(200));

        // Attempt 2: 400ms
        let delay2 = config.delay_for_attempt(2);
        assert_eq!(delay2, Duration::from_millis(400));

        // Should cap at max_delay
        let delay_large = config.delay_for_attempt(20);
        assert!(delay_large <= config.max_delay);
    }

    #[tokio::test]
    async fn test_reconnect_client_creation() {
        // Try to connect to non-existent server
        let config = ReconnectConfig::with_max_attempts(1);
        let result = ReconnectClient::connect("127.0.0.1:19999", config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reconnect_count() {
        let config = ReconnectConfig::default();

        // Create a listener for the client to connect to
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            // Accept connections but don't do anything
            loop {
                let _ = listener.accept().await;
            }
        });

        let client = ReconnectClient::connect(&addr.to_string(), config).await.unwrap();
        assert_eq!(client.reconnect_count(), 0);
    }
}
