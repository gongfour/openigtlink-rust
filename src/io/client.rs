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
}

#[cfg(test)]
mod tests {
    // Integration tests require a real server
    // See examples/ for end-to-end testing
}
