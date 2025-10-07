//! Asynchronous OpenIGTLink server implementation
//!
//! Provides a non-blocking, async/await-based server for OpenIGTLink communication.

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, trace, warn};

/// Asynchronous OpenIGTLink server
///
/// Uses non-blocking I/O with Tokio for high-concurrency scenarios.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::AsyncIgtlServer;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let server = AsyncIgtlServer::bind("127.0.0.1:18944").await?;
///     let connection = server.accept().await?;
///     Ok(())
/// }
/// ```
pub struct AsyncIgtlServer {
    listener: TcpListener,
}

impl AsyncIgtlServer {
    /// Bind to a local address and create a server asynchronously
    ///
    /// # Arguments
    ///
    /// * `addr` - Local address to bind (e.g., "127.0.0.1:18944")
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to bind
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::AsyncIgtlServer;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let server = AsyncIgtlServer::bind("127.0.0.1:18944").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn bind(addr: &str) -> Result<Self> {
        info!(addr = %addr, "Binding OpenIGTLink server (async)");
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;
        info!(
            local_addr = %local_addr,
            "OpenIGTLink server listening (async)"
        );
        Ok(AsyncIgtlServer { listener })
    }

    /// Accept a new client connection asynchronously
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to accept connection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::AsyncIgtlServer;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let server = AsyncIgtlServer::bind("127.0.0.1:18944").await?;
    ///     let connection = server.accept().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn accept(&self) -> Result<AsyncIgtlConnection> {
        trace!("Waiting for client connection (async)");
        let (stream, addr) = self.listener.accept().await?;
        info!(
            peer_addr = %addr,
            "Client connected (async)"
        );
        Ok(AsyncIgtlConnection {
            stream,
            verify_crc: true,
        })
    }

    /// Get the local address this server is bound to
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.listener.local_addr()?)
    }
}

/// Represents an accepted client connection (async)
///
/// Provides methods to send and receive OpenIGTLink messages asynchronously.
pub struct AsyncIgtlConnection {
    stream: TcpStream,
    verify_crc: bool,
}

impl AsyncIgtlConnection {
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

    /// Send a message to the connected client asynchronously
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
    /// use openigtlink_rust::io::AsyncIgtlServer;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let server = AsyncIgtlServer::bind("127.0.0.1:18944").await?;
    ///     let mut conn = server.accept().await?;
    ///
    ///     let status = StatusMessage::ok("Ready");
    ///     let msg = IgtlMessage::new(status, "Server")?;
    ///     conn.send(&msg).await?;
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
            "Sending message to client (async)"
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

    /// Receive a message from the connected client asynchronously
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
    /// use openigtlink_rust::io::AsyncIgtlServer;
    /// use openigtlink_rust::protocol::types::TransformMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let server = AsyncIgtlServer::bind("127.0.0.1:18944").await?;
    ///     let mut conn = server.accept().await?;
    ///
    ///     let msg: IgtlMessage<TransformMessage> = conn.receive().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        trace!("Waiting for message header from client (async)");

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
            "Received message header from client (async)"
        );

        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf).await?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received from client (async)"
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
                    "Failed to decode message from client (async)"
                );
            }
        }

        result
    }

    /// Enable or disable TCP_NODELAY (Nagle's algorithm)
    pub async fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.stream.set_nodelay(nodelay)?;
        debug!(nodelay = nodelay, "TCP_NODELAY configured (async)");
        Ok(())
    }

    /// Get the current TCP_NODELAY setting
    pub async fn nodelay(&self) -> Result<bool> {
        Ok(self.stream.nodelay()?)
    }

    /// Get the remote peer address
    pub fn peer_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.stream.peer_addr()?)
    }

    /// Split the connection into read and write halves
    ///
    /// This allows concurrent reading and writing on separate tasks.
    pub fn into_split(self) -> (AsyncIgtlConnectionReader, AsyncIgtlConnectionWriter) {
        let (reader, writer) = self.stream.into_split();
        (
            AsyncIgtlConnectionReader {
                reader,
                verify_crc: self.verify_crc,
            },
            AsyncIgtlConnectionWriter { writer },
        )
    }
}

/// Read half of an async OpenIGTLink connection
pub struct AsyncIgtlConnectionReader {
    reader: tokio::net::tcp::OwnedReadHalf,
    verify_crc: bool,
}

impl AsyncIgtlConnectionReader {
    /// Receive a message from the read half
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        trace!("Waiting for message header (async connection reader)");

        let mut header_buf = vec![0u8; Header::SIZE];
        self.reader.read_exact(&mut header_buf).await?;

        let header = Header::decode(&header_buf)?;

        let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            body_size = header.body_size,
            "Received message header (async connection reader)"
        );

        let mut body_buf = vec![0u8; header.body_size as usize];
        self.reader.read_exact(&mut body_buf).await?;

        let mut full_msg = header_buf;
        full_msg.extend_from_slice(&body_buf);

        IgtlMessage::decode_with_options(&full_msg, self.verify_crc)
    }
}

/// Write half of an async OpenIGTLink connection
pub struct AsyncIgtlConnectionWriter {
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl AsyncIgtlConnectionWriter {
    /// Send a message to the write half
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            size = data.len(),
            "Sending message (async connection writer)"
        );

        self.writer.write_all(&data).await?;
        self.writer.flush().await?;

        trace!(
            msg_type = msg_type,
            bytes_sent = data.len(),
            "Message sent (async connection writer)"
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
    async fn test_async_server_bind() {
        let server = AsyncIgtlServer::bind("127.0.0.1:0").await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_async_server_local_addr() {
        let server = AsyncIgtlServer::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap();
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
    }

    #[tokio::test]
    async fn test_async_server_client_communication() {
        // Create server
        let server = AsyncIgtlServer::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap();

        // Spawn server task
        tokio::spawn(async move {
            let mut conn = server.accept().await.unwrap();

            // Receive message
            let msg: IgtlMessage<StatusMessage> = conn.receive().await.unwrap();
            assert_eq!(msg.content.status_string, "Hello from client");

            // Send response
            let response = StatusMessage::ok("Hello from server");
            let response_msg = IgtlMessage::new(response, "Server").unwrap();
            conn.send(&response_msg).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        // Connect client
        use crate::io::AsyncIgtlClient;
        let mut client = AsyncIgtlClient::connect(&addr.to_string())
            .await
            .unwrap();

        // Send message
        let status = StatusMessage::ok("Hello from client");
        let msg = IgtlMessage::new(status, "Client").unwrap();
        client.send(&msg).await.unwrap();

        // Receive response
        let response: IgtlMessage<StatusMessage> = client.receive().await.unwrap();
        assert_eq!(response.content.status_string, "Hello from server");
    }

    #[tokio::test]
    async fn test_async_connection_split() {
        let server = AsyncIgtlServer::bind("127.0.0.1:0").await.unwrap();
        let addr = server.local_addr().unwrap();

        tokio::spawn(async move {
            let conn = server.accept().await.unwrap();
            let (mut reader, mut writer) = conn.into_split();

            // Receive and echo back
            let msg: IgtlMessage<StatusMessage> = reader.receive().await.unwrap();
            let echo = IgtlMessage::new(msg.content, "Echo").unwrap();
            writer.send(&echo).await.unwrap();
        });

        tokio::time::sleep(Duration::from_millis(10)).await;

        use crate::io::AsyncIgtlClient;
        let mut client = AsyncIgtlClient::connect(&addr.to_string())
            .await
            .unwrap();

        let status = StatusMessage::ok("Echo test");
        let msg = IgtlMessage::new(status, "Client").unwrap();
        client.send(&msg).await.unwrap();

        let response: IgtlMessage<StatusMessage> = client.receive().await.unwrap();
        assert_eq!(response.content.status_string, "Echo test");
    }
}
