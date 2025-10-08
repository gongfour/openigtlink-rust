//! Synchronous OpenIGTLink client
//!
//! Simple blocking TCP client for OpenIGTLink communication.

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use tracing::{debug, info, trace};

/// Synchronous OpenIGTLink client
///
/// Uses blocking I/O with `std::net::TcpStream` for simple, synchronous communication.
///
/// **Recommended**: Use [`ClientBuilder`](crate::io::builder::ClientBuilder) for better ergonomics:
/// ```no_run
/// use openigtlink_rust::io::builder::ClientBuilder;
/// let client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;
/// # Ok::<(), openigtlink_rust::error::IgtlError>(())
/// ```
pub struct SyncTcpClient {
    stream: TcpStream,
    verify_crc: bool,
}

impl SyncTcpClient {
    /// Connect to an OpenIGTLink server
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "127.0.0.1:18944")
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // This is an internal type. Use ClientBuilder instead.
    /// use openigtlink_rust::io::sync_client::SyncTcpClient;
    ///
    /// let client = SyncTcpClient::connect("127.0.0.1:18944")?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn connect(addr: &str) -> Result<Self> {
        info!("Connecting to {}", addr);
        let stream = TcpStream::connect(addr)?;
        debug!("Connected to {}", addr);

        Ok(SyncTcpClient {
            stream,
            verify_crc: true,
        })
    }

    /// Enable or disable CRC verification for received messages
    ///
    /// # Arguments
    ///
    /// * `verify` - true to enable CRC verification, false to disable
    pub fn set_verify_crc(&mut self, verify: bool) {
        self.verify_crc = verify;
    }

    /// Get current CRC verification setting
    pub fn verify_crc(&self) -> bool {
        self.verify_crc
    }

    /// Set read timeout for receive operations
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration (None for blocking forever)
    pub fn set_read_timeout(&self, timeout: Option<std::time::Duration>) -> Result<()> {
        self.stream.set_read_timeout(timeout)?;
        Ok(())
    }

    /// Set write timeout for send operations
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration (None for blocking forever)
    pub fn set_write_timeout(&self, timeout: Option<std::time::Duration>) -> Result<()> {
        self.stream.set_write_timeout(timeout)?;
        Ok(())
    }

    /// Send a message to the server
    ///
    /// # Arguments
    ///
    /// * `msg` - Message to send
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::builder::ClientBuilder;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let mut client = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .sync()
    ///     .build()?;
    ///
    /// let status = StatusMessage::ok("Hello");
    /// let msg = IgtlMessage::new(status, "MyDevice")?;
    /// // client.send(&msg)?;  // Requires unified client API
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        trace!("Sending {} bytes", data.len());

        self.stream.write_all(&data)?;
        self.stream.flush()?;

        debug!("Sent {} bytes", data.len());
        Ok(())
    }

    /// Receive a message from the server
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::builder::ClientBuilder;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let mut client = ClientBuilder::new()
    ///     .tcp("127.0.0.1:18944")
    ///     .sync()
    ///     .build()?;
    ///
    /// // let msg: IgtlMessage<StatusMessage> = client.receive()?;  // Requires unified client API
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        // Read header (58 bytes)
        let mut header_buf = [0u8; 58];
        self.stream.read_exact(&mut header_buf)?;

        let header = Header::decode(&header_buf)?;
        debug!("Received header: size={}", header.body_size);

        // Read body
        let body_size = header.body_size as usize;
        let mut body_buf = vec![0u8; body_size];
        self.stream.read_exact(&mut body_buf)?;

        // Combine header and body
        let mut full_msg = header_buf.to_vec();
        full_msg.extend_from_slice(&body_buf);

        // Decode message
        let result = IgtlMessage::decode_with_options(&full_msg, self.verify_crc);

        match &result {
            Ok(_msg) => {
                trace!("Successfully decoded message");
            }
            Err(e) => {
                debug!("Failed to decode message: {}", e);
            }
        }

        result
    }
}
