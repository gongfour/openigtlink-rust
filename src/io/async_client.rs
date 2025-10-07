//! Asynchronous OpenIGTLink client implementation
//!
//! Provides a non-blocking, async/await-based client for OpenIGTLink communication.
//! Uses Tokio for async I/O operations.

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, trace, warn};

/// Asynchronous OpenIGTLink client
///
/// Uses non-blocking I/O with Tokio for high-concurrency scenarios.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::AsyncIgtlClient;
/// use openigtlink_rust::protocol::types::StatusMessage;
/// use openigtlink_rust::protocol::message::IgtlMessage;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = AsyncIgtlClient::connect("127.0.0.1:18944").await?;
///
///     let status = StatusMessage::ok("Hello");
///     let msg = IgtlMessage::new(status, "AsyncClient")?;
///     client.send(&msg).await?;
///
///     Ok(())
/// }
/// ```
pub struct AsyncIgtlClient {
    stream: TcpStream,
    verify_crc: bool,
}

impl AsyncIgtlClient {
    /// Connect to an OpenIGTLink server asynchronously
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "127.0.0.1:18944")
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to connect
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::AsyncIgtlClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = AsyncIgtlClient::connect("127.0.0.1:18944").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn connect(addr: &str) -> Result<Self> {
        info!(addr = %addr, "Connecting to OpenIGTLink server (async)");
        let stream = TcpStream::connect(addr).await?;
        let local_addr = stream.local_addr()?;
        info!(
            local_addr = %local_addr,
            remote_addr = %addr,
            "Connected to OpenIGTLink server (async)"
        );
        Ok(AsyncIgtlClient {
            stream,
            verify_crc: true,
        })
    }

    /// Enable or disable CRC verification for received messages
    ///
    /// # Arguments
    ///
    /// * `verify` - true to enable CRC verification (default), false to disable
    ///
    /// # Safety
    ///
    /// Disabling CRC verification should only be done in trusted environments
    /// where data corruption is unlikely (e.g., loopback, local network).
    pub fn set_verify_crc(&mut self, verify: bool) {
        if verify != self.verify_crc {
            info!(verify = verify, "CRC verification setting changed");
            if !verify {
                warn!("CRC verification disabled - use only in trusted environments");
            }
        }
        self.verify_crc = verify;
    }

    /// Get current CRC verification setting
    pub fn verify_crc(&self) -> bool {
        self.verify_crc
    }

    /// Send a message to the server asynchronously
    ///
    /// # Arguments
    ///
    /// * `msg` - Message to send
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network write failed
    /// - [`IgtlError::BodyTooLarge`](crate::error::IgtlError::BodyTooLarge) - Message exceeds maximum size
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::AsyncIgtlClient;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = AsyncIgtlClient::connect("127.0.0.1:18944").await?;
    ///
    ///     let status = StatusMessage::ok("Ready");
    ///     let msg = IgtlMessage::new(status, "Client")?;
    ///     client.send(&msg).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            size = data.len(),
            "Sending message (async)"
        );

        self.stream.write_all(&data).await?;
        self.stream.flush().await?;

        trace!(
            msg_type = msg_type,
            bytes_sent = data.len(),
            "Message sent successfully (async)"
        );

        Ok(())
    }

    /// Receive a message from the server asynchronously
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network read failed
    /// - [`IgtlError::InvalidHeader`](crate::error::IgtlError::InvalidHeader) - Received malformed header
    /// - [`IgtlError::CrcMismatch`](crate::error::IgtlError::CrcMismatch) - Data corruption detected
    /// - [`IgtlError::UnknownMessageType`](crate::error::IgtlError::UnknownMessageType) - Unsupported message type
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::AsyncIgtlClient;
    /// use openigtlink_rust::protocol::types::TransformMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut client = AsyncIgtlClient::connect("127.0.0.1:18944").await?;
    ///     let msg: IgtlMessage<TransformMessage> = client.receive().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        trace!("Waiting for message header (async)");

        let mut header_buf = vec![0u8; Header::SIZE];
        self.stream.read_exact(&mut header_buf).await?;

        let header = Header::decode(&header_buf)?;

        let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            body_size = header.body_size,
            version = header.version,
            "Received message header (async)"
        );

        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf).await?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received (async)"
        );

        let mut full_msg = header_buf;
        full_msg.extend_from_slice(&body_buf);

        let result = IgtlMessage::decode_with_options(&full_msg, self.verify_crc);

        match &result {
            Ok(_) => {
                debug!(
                    msg_type = msg_type,
                    device_name = device_name,
                    "Message decoded successfully (async)"
                );
            }
            Err(e) => {
                warn!(
                    msg_type = msg_type,
                    error = %e,
                    "Failed to decode message (async)"
                );
            }
        }

        result
    }

    /// Set read timeout for the underlying TCP stream
    pub async fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result<()> {
        // Tokio doesn't use set_read_timeout, instead use tokio::time::timeout
        // This is a placeholder for API compatibility
        debug!(timeout_ms = ?timeout.map(|d| d.as_millis()), "Read timeout not directly supported in async (use tokio::time::timeout)");
        Ok(())
    }

    /// Set write timeout for the underlying TCP stream
    pub async fn set_write_timeout(
        &mut self,
        timeout: Option<std::time::Duration>,
    ) -> Result<()> {
        // Tokio doesn't use set_write_timeout, instead use tokio::time::timeout
        // This is a placeholder for API compatibility
        debug!(timeout_ms = ?timeout.map(|d| d.as_millis()), "Write timeout not directly supported in async (use tokio::time::timeout)");
        Ok(())
    }

    /// Enable or disable TCP_NODELAY (Nagle's algorithm)
    pub async fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.stream.set_nodelay(nodelay)?;
        debug!(nodelay = nodelay, "TCP_NODELAY configured");
        Ok(())
    }

    /// Get the current TCP_NODELAY setting
    pub async fn nodelay(&self) -> Result<bool> {
        Ok(self.stream.nodelay()?)
    }

    /// Get the local address
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.stream.local_addr()?)
    }

    /// Get the remote peer address
    pub fn peer_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.stream.peer_addr()?)
    }

    /// Split the client into read and write halves
    ///
    /// This allows concurrent reading and writing on separate tasks.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::AsyncIgtlClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = AsyncIgtlClient::connect("127.0.0.1:18944").await?;
    ///     let (reader, writer) = client.into_split();
    ///
    ///     // Use reader and writer in separate tasks
    ///     Ok(())
    /// }
    /// ```
    pub fn into_split(self) -> (AsyncIgtlReader, AsyncIgtlWriter) {
        let (reader, writer) = self.stream.into_split();
        (
            AsyncIgtlReader {
                reader,
                verify_crc: self.verify_crc,
            },
            AsyncIgtlWriter { writer },
        )
    }
}

/// Read half of an async OpenIGTLink client
pub struct AsyncIgtlReader {
    reader: tokio::net::tcp::OwnedReadHalf,
    verify_crc: bool,
}

impl AsyncIgtlReader {
    /// Receive a message from the read half
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        trace!("Waiting for message header (async reader)");

        let mut header_buf = vec![0u8; Header::SIZE];
        self.reader.read_exact(&mut header_buf).await?;

        let header = Header::decode(&header_buf)?;

        let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            body_size = header.body_size,
            "Received message header (async reader)"
        );

        let mut body_buf = vec![0u8; header.body_size as usize];
        self.reader.read_exact(&mut body_buf).await?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received (async reader)"
        );

        let mut full_msg = header_buf;
        full_msg.extend_from_slice(&body_buf);

        IgtlMessage::decode_with_options(&full_msg, self.verify_crc)
    }
}

/// Write half of an async OpenIGTLink client
pub struct AsyncIgtlWriter {
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl AsyncIgtlWriter {
    /// Send a message to the write half
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            size = data.len(),
            "Sending message (async writer)"
        );

        self.writer.write_all(&data).await?;
        self.writer.flush().await?;

        trace!(
            msg_type = msg_type,
            bytes_sent = data.len(),
            "Message sent (async writer)"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::StatusMessage;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_async_client_connect_timeout() {
        // Try to connect to a non-existent server with timeout
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            AsyncIgtlClient::connect("127.0.0.1:19999"),
        )
        .await;

        // Should timeout or fail to connect
        assert!(result.is_err() || result.unwrap().is_err());
    }

    #[tokio::test]
    async fn test_async_client_crc_setting() {
        // Test CRC setting without actual connection
        let stream = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap()
            .local_addr()
            .unwrap();

        tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(stream).await.unwrap();
            let _ = listener.accept().await;
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        let mut client = AsyncIgtlClient::connect(&stream.to_string())
            .await
            .unwrap();
        assert_eq!(client.verify_crc(), true);

        client.set_verify_crc(false);
        assert_eq!(client.verify_crc(), false);

        client.set_verify_crc(true);
        assert_eq!(client.verify_crc(), true);
    }

    #[tokio::test]
    async fn test_async_client_server_communication() {
        // Create server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server task
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut client = AsyncIgtlClient {
                stream,
                verify_crc: true,
            };

            // Receive message
            let msg: IgtlMessage<StatusMessage> = client.receive().await.unwrap();
            assert_eq!(msg.content.status_string, "Hello");

            // Send response
            let response = StatusMessage::ok("World");
            let response_msg = IgtlMessage::new(response, "Server").unwrap();
            client.send(&response_msg).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Connect client
        let mut client = AsyncIgtlClient::connect(&addr.to_string())
            .await
            .unwrap();

        // Send message
        let status = StatusMessage::ok("Hello");
        let msg = IgtlMessage::new(status, "Client").unwrap();
        client.send(&msg).await.unwrap();

        // Receive response
        let response: IgtlMessage<StatusMessage> = client.receive().await.unwrap();
        assert_eq!(response.content.status_string, "World");
    }
}
