//! Synchronous OpenIGTLink client implementation
//!
//! Provides a simple blocking TCP client for OpenIGTLink communication.

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};

/// Synchronous OpenIGTLink client
///
/// Uses blocking I/O with `std::net::TcpStream` for simple, synchronous communication.
pub struct IgtlClient {
    stream: TcpStream,
}

impl IgtlClient {
    /// Connect to an OpenIGTLink server
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (e.g., "127.0.0.1:18944")
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Connection failed (server not running, network unreachable, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let client = IgtlClient::connect("127.0.0.1:18944")?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr)?;
        Ok(IgtlClient { stream })
    }

    /// Send a message to the server
    ///
    /// # Arguments
    ///
    /// * `msg` - Message to send
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network write failed (connection lost, broken pipe, etc.)
    /// - [`IgtlError::BodyTooLarge`](crate::error::IgtlError::BodyTooLarge) - Message exceeds maximum size
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    /// use openigtlink_rust::protocol::types::TransformMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    /// let transform = TransformMessage::identity();
    /// let msg = IgtlMessage::new(transform, "Device1")?;
    /// client.send(&msg)?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        self.stream.write_all(&data)?;
        self.stream.flush()?;
        Ok(())
    }

    /// Receive a message from the server
    ///
    /// Blocks until a complete message is received.
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network read failed (connection lost, timeout, etc.)
    /// - [`IgtlError::InvalidHeader`](crate::error::IgtlError::InvalidHeader) - Received malformed header
    /// - [`IgtlError::CrcMismatch`](crate::error::IgtlError::CrcMismatch) - Data corruption detected
    /// - [`IgtlError::UnknownMessageType`](crate::error::IgtlError::UnknownMessageType) - Unsupported message type
    /// - [`IgtlError::InvalidSize`](crate::error::IgtlError::InvalidSize) - Message size mismatch
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    /// let msg: IgtlMessage<StatusMessage> = client.receive()?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        // Read header (58 bytes)
        let mut header_buf = vec![0u8; Header::SIZE];
        self.stream.read_exact(&mut header_buf)?;

        let header = Header::decode(&header_buf)?;

        // Read body
        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf)?;

        // Decode full message
        let mut full_msg = header_buf;
        full_msg.extend_from_slice(&body_buf);

        IgtlMessage::decode(&full_msg)
    }

    /// Set read timeout for the underlying TCP stream
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration (None for infinite)
    pub fn set_read_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result<()> {
        self.stream.set_read_timeout(timeout)?;
        Ok(())
    }

    /// Set write timeout for the underlying TCP stream
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration (None for infinite)
    pub fn set_write_timeout(&mut self, timeout: Option<std::time::Duration>) -> Result<()> {
        self.stream.set_write_timeout(timeout)?;
        Ok(())
    }

    /// Request server capabilities (convenience method)
    ///
    /// Sends a GET_CAPABIL query and receives the CAPABILITY response.
    ///
    /// # Returns
    ///
    /// `CapabilityMessage` containing the list of supported message types
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network communication failed
    /// - Other errors from send/receive operations
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let mut client = IgtlClient::connect("192.168.1.100:18944")?;
    /// let capability = client.request_capability()?;
    ///
    /// println!("Server supports {} message types:", capability.types.len());
    /// for msg_type in &capability.types {
    ///     println!("  - {}", msg_type);
    /// }
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn request_capability(&mut self) -> Result<crate::protocol::types::CapabilityMessage> {
        use crate::protocol::types::GetCapabilityMessage;

        let query = GetCapabilityMessage;
        let msg = IgtlMessage::new(query, "Client")?;
        self.send(&msg)?;

        let response: IgtlMessage<crate::protocol::types::CapabilityMessage> = self.receive()?;
        Ok(response.content)
    }

    /// Start tracking data stream (convenience method)
    ///
    /// Sends a STT_TDATA message to request tracking data streaming and waits for
    /// the server's RTS_TDATA acknowledgment.
    ///
    /// # Arguments
    ///
    /// * `resolution` - Update interval in milliseconds (e.g., 50ms = 20 Hz)
    /// * `coordinate_name` - Coordinate system name (e.g., "RAS", "LPS", max 32 bytes)
    ///
    /// # Returns
    ///
    /// `RtsTDataMessage` containing the server's status (0=error, 1=ok)
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network communication failed
    /// - [`IgtlError::InvalidHeader`](crate::error::IgtlError::InvalidHeader) - Server rejected request (status=0)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    /// use openigtlink_rust::protocol::types::TDataMessage;
    ///
    /// let mut client = IgtlClient::connect("192.168.1.100:18944")?;
    ///
    /// // Start 20 Hz tracking stream in RAS coordinate system
    /// let ack = client.start_tracking(50, "RAS")?;
    ///
    /// if ack.status == 1 {
    ///     // Server ready, receive tracking data
    ///     for _ in 0..100 {
    ///         let tdata: openigtlink_rust::protocol::message::IgtlMessage<TDataMessage> =
    ///             client.receive()?;
    ///         // Process tracking data...
    ///     }
    ///
    ///     // Stop streaming
    ///     client.stop_tracking()?;
    /// }
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn start_tracking(
        &mut self,
        resolution: u32,
        coordinate_name: &str,
    ) -> Result<crate::protocol::types::RtsTDataMessage> {
        use crate::protocol::types::StartTDataMessage;

        let start = StartTDataMessage {
            resolution,
            coordinate_name: coordinate_name.to_string(),
        };

        let msg = IgtlMessage::new(start, "Client")?;
        self.send(&msg)?;

        let response: IgtlMessage<crate::protocol::types::RtsTDataMessage> = self.receive()?;
        Ok(response.content)
    }

    /// Stop tracking data stream (convenience method)
    ///
    /// Sends a STP_TDATA message to request the server to stop streaming tracking data.
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network communication failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let mut client = IgtlClient::connect("192.168.1.100:18944")?;
    ///
    /// // ... streaming session ...
    ///
    /// // Stop streaming when done
    /// client.stop_tracking()?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn stop_tracking(&mut self) -> Result<()> {
        use crate::protocol::types::StopTDataMessage;

        let stop = StopTDataMessage;
        let msg = IgtlMessage::new(stop, "Client")?;
        self.send(&msg)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require a real server
    // See examples/ for end-to-end testing
}
