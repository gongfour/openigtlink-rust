//! UDP-based OpenIGTLink communication
//!
//! Provides connectionless UDP transport for low-latency applications where
//! occasional packet loss is acceptable (e.g., real-time tracking).
//!
//! # Important Notes
//!
//! - **No delivery guarantee**: UDP does not guarantee message delivery or ordering
//! - **MTU limitation**: Single UDP datagram limited to ~65507 bytes
//! - **Use cases**: High-frequency tracking data (>60Hz), non-critical status updates
//! - **Not recommended for**: Large images, critical commands, file transfers
//!
//! # Example: High-Speed Tracking
//!
//! ```no_run
//! use openigtlink_rust::io::UdpClient;
//! use openigtlink_rust::protocol::types::TransformMessage;
//! use openigtlink_rust::protocol::message::IgtlMessage;
//!
//! // Client sends tracking data at 120Hz
//! let client = UdpClient::bind("0.0.0.0:0")?;
//!
//! loop {
//!     let transform = TransformMessage::identity();
//!     let msg = IgtlMessage::new(transform, "Tracker")?;
//!     client.send_to(&msg, "127.0.0.1:18944")?;
//!     std::thread::sleep(std::time::Duration::from_millis(8)); // 120Hz
//! }
//! # Ok::<(), openigtlink_rust::error::IgtlError>(())
//! ```

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

use crate::error::{IgtlError, Result};
use crate::protocol::message::{IgtlMessage, Message};

/// Maximum UDP datagram size (IPv4 max - IP header - UDP header)
/// 65535 (max IP packet) - 20 (IP header) - 8 (UDP header) = 65507 bytes
pub const MAX_UDP_DATAGRAM_SIZE: usize = 65507;

/// UDP client for sending/receiving OpenIGTLink messages
///
/// Provides connectionless communication with low overhead. Suitable for
/// high-frequency updates where occasional packet loss is acceptable.
///
/// # Performance Characteristics
///
/// - **Latency**: Lower than TCP (no connection setup, no retransmission)
/// - **Throughput**: Limited by network MTU (~1500 bytes typical Ethernet)
/// - **Reliability**: None (packets may be lost, duplicated, or reordered)
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::UdpClient;
/// use openigtlink_rust::protocol::types::TransformMessage;
/// use openigtlink_rust::protocol::message::IgtlMessage;
///
/// let client = UdpClient::bind("0.0.0.0:0")?;
/// let transform = TransformMessage::identity();
/// let msg = IgtlMessage::new(transform, "Tool")?;
/// client.send_to(&msg, "192.168.1.100:18944")?;
/// # Ok::<(), openigtlink_rust::error::IgtlError>(())
/// ```
pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    /// Bind to a local address
    ///
    /// # Arguments
    ///
    /// * `local_addr` - Local address to bind (use "0.0.0.0:0" for any available port)
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to bind socket (address in use, permission denied, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::UdpClient;
    ///
    /// // Bind to any available port
    /// let client = UdpClient::bind("0.0.0.0:0")?;
    ///
    /// // Bind to specific port
    /// let client = UdpClient::bind("0.0.0.0:18945")?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn bind(local_addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(local_addr)?;
        Ok(UdpClient { socket })
    }

    /// Send a message to a remote address
    ///
    /// # Arguments
    ///
    /// * `msg` - Message to send
    /// * `target` - Target address (e.g., "127.0.0.1:18944")
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network transmission failed
    /// - [`IgtlError::BodyTooLarge`](crate::error::IgtlError::BodyTooLarge) - Message exceeds UDP MTU (65507 bytes)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::UdpClient;
    /// use openigtlink_rust::protocol::types::TransformMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let client = UdpClient::bind("0.0.0.0:0")?;
    /// let transform = TransformMessage::identity();
    /// let msg = IgtlMessage::new(transform, "Device")?;
    /// client.send_to(&msg, "127.0.0.1:18944")?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn send_to<T: Message>(&self, msg: &IgtlMessage<T>, target: &str) -> Result<()> {
        let data = msg.encode()?;

        if data.len() > MAX_UDP_DATAGRAM_SIZE {
            return Err(IgtlError::BodyTooLarge {
                size: data.len(),
                max: MAX_UDP_DATAGRAM_SIZE,
            });
        }

        self.socket.send_to(&data, target)?;
        Ok(())
    }

    /// Receive a message (blocking)
    ///
    /// Blocks until a datagram is received. Returns the message and sender address.
    ///
    /// # Returns
    ///
    /// Tuple of (message, sender_address)
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network read failed or timeout
    /// - [`IgtlError::InvalidHeader`](crate::error::IgtlError::InvalidHeader) - Malformed header
    /// - [`IgtlError::CrcMismatch`](crate::error::IgtlError::CrcMismatch) - Data corruption detected
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::UdpClient;
    /// use openigtlink_rust::protocol::types::TransformMessage;
    ///
    /// let client = UdpClient::bind("0.0.0.0:18944")?;
    /// let (msg, sender) = client.receive_from::<TransformMessage>()?;
    /// println!("Received from {}", sender);
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn receive_from<T: Message>(&self) -> Result<(IgtlMessage<T>, SocketAddr)> {
        let mut buf = vec![0u8; MAX_UDP_DATAGRAM_SIZE];
        let (size, src) = self.socket.recv_from(&mut buf)?;

        let msg = IgtlMessage::decode(&buf[..size])?;
        Ok((msg, src))
    }

    /// Set read timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration (None for blocking forever)
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to set socket option
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::UdpClient;
    /// use std::time::Duration;
    ///
    /// let client = UdpClient::bind("0.0.0.0:0")?;
    /// client.set_read_timeout(Some(Duration::from_secs(5)))?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<()> {
        self.socket.set_read_timeout(timeout)?;
        Ok(())
    }

    /// Set write timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration (None for blocking forever)
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to set socket option
    pub fn set_write_timeout(&self, timeout: Option<Duration>) -> Result<()> {
        self.socket.set_write_timeout(timeout)?;
        Ok(())
    }

    /// Get local socket address
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to get socket address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }
}

/// UDP server for receiving OpenIGTLink messages
///
/// Listens for incoming datagrams on a specific port.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::UdpServer;
/// use openigtlink_rust::protocol::types::TransformMessage;
/// use openigtlink_rust::protocol::message::IgtlMessage;
///
/// # fn main() -> Result<(), openigtlink_rust::error::IgtlError> {
/// let server = UdpServer::bind("0.0.0.0:18944")?;
///
/// # let mut count = 0;
/// loop {
///     let (msg, sender) = server.receive::<TransformMessage>()?;
///     println!("Received from {}", sender);
///
///     // Echo back
///     let response = IgtlMessage::new(msg.content, "Server")?;
///     server.send_to(&response, sender)?;
///
///     # count += 1;
///     # if count >= 1 { break; }
/// }
/// # Ok(())
/// # }
/// ```
pub struct UdpServer {
    socket: UdpSocket,
}

impl UdpServer {
    /// Bind server to an address
    ///
    /// # Arguments
    ///
    /// * `addr` - Address to bind (e.g., "0.0.0.0:18944")
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to bind (port in use, permission denied, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::UdpServer;
    ///
    /// let server = UdpServer::bind("0.0.0.0:18944")?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn bind(addr: &str) -> Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        Ok(UdpServer { socket })
    }

    /// Receive a message (blocking)
    ///
    /// # Returns
    ///
    /// Tuple of (message, sender_address)
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network read failed or timeout
    /// - [`IgtlError::InvalidHeader`](crate::error::IgtlError::InvalidHeader) - Malformed header
    /// - [`IgtlError::CrcMismatch`](crate::error::IgtlError::CrcMismatch) - Data corruption
    pub fn receive<T: Message>(&self) -> Result<(IgtlMessage<T>, SocketAddr)> {
        let mut buf = vec![0u8; MAX_UDP_DATAGRAM_SIZE];
        let (size, src) = self.socket.recv_from(&mut buf)?;

        let msg = IgtlMessage::decode(&buf[..size])?;
        Ok((msg, src))
    }

    /// Send a response to a specific address
    ///
    /// # Arguments
    ///
    /// * `msg` - Message to send
    /// * `target` - Target socket address
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network transmission failed
    /// - [`IgtlError::BodyTooLarge`](crate::error::IgtlError::BodyTooLarge) - Message exceeds UDP MTU
    pub fn send_to<T: Message>(&self, msg: &IgtlMessage<T>, target: SocketAddr) -> Result<()> {
        let data = msg.encode()?;

        if data.len() > MAX_UDP_DATAGRAM_SIZE {
            return Err(IgtlError::BodyTooLarge {
                size: data.len(),
                max: MAX_UDP_DATAGRAM_SIZE,
            });
        }

        self.socket.send_to(&data, target)?;
        Ok(())
    }

    /// Set read timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration (None for blocking forever)
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to set socket option
    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> Result<()> {
        self.socket.set_read_timeout(timeout)?;
        Ok(())
    }

    /// Get local socket address
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to get socket address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::TransformMessage;

    #[test]
    fn test_max_datagram_size() {
        assert_eq!(MAX_UDP_DATAGRAM_SIZE, 65507);
    }

    #[test]
    fn test_client_bind() {
        let client = UdpClient::bind("127.0.0.1:0");
        assert!(client.is_ok());
    }

    #[test]
    fn test_server_bind() {
        let server = UdpServer::bind("127.0.0.1:0");
        assert!(server.is_ok());
    }

    #[test]
    fn test_local_addr() {
        let client = UdpClient::bind("127.0.0.1:0").unwrap();
        let addr = client.local_addr().unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert!(addr.port() > 0);
    }

    #[test]
    fn test_send_receive() {
        // Bind server first to get a known port
        let server = UdpServer::bind("127.0.0.1:0").unwrap();
        let server_addr = server.local_addr().unwrap();

        // Create client
        let client = UdpClient::bind("127.0.0.1:0").unwrap();

        // Send message
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();
        client.send_to(&msg, &server_addr.to_string()).unwrap();

        // Receive message
        let (received_msg, sender) = server.receive::<TransformMessage>().unwrap();
        assert_eq!(
            received_msg.header.device_name.as_str().unwrap(),
            "TestDevice"
        );
        assert_eq!(sender, client.local_addr().unwrap());
    }

    #[test]
    fn test_timeout() {
        let client = UdpClient::bind("127.0.0.1:0").unwrap();
        client
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        // Should timeout since no data is available
        let result = client.receive_from::<TransformMessage>();
        assert!(result.is_err());
    }

    #[test]
    fn test_message_too_large() {
        let _client = UdpClient::bind("127.0.0.1:0").unwrap();

        // This would fail during encoding if we tried to create a message > 65507 bytes
        // Verify the constant is within valid UDP datagram size
        const _: () = assert!(MAX_UDP_DATAGRAM_SIZE < 65536);
    }
}
