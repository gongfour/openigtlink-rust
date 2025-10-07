//! Synchronous OpenIGTLink server implementation
//!
//! Provides a simple blocking TCP server for OpenIGTLink communication.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};

/// Synchronous OpenIGTLink server
///
/// Uses blocking I/O with `std::net::TcpListener` for simple, synchronous server implementation.
pub struct IgtlServer {
    listener: TcpListener,
}

impl IgtlServer {
    /// Bind to a local address and create a server
    ///
    /// # Arguments
    ///
    /// * `addr` - Local address to bind (e.g., "127.0.0.1:18944")
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to bind (address in use, insufficient permissions, etc.)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlServer;
    ///
    /// let server = IgtlServer::bind("127.0.0.1:18944")?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn bind(addr: &str) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(IgtlServer { listener })
    }

    /// Accept a new client connection
    ///
    /// Blocks until a client connects.
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to accept connection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlServer;
    ///
    /// let server = IgtlServer::bind("127.0.0.1:18944")?;
    /// let connection = server.accept()?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn accept(&self) -> Result<IgtlConnection> {
        let (stream, _addr) = self.listener.accept()?;
        Ok(IgtlConnection {
            stream,
            verify_crc: true, // Default: verify CRC
        })
    }

    /// Get the local address this server is bound to
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.listener.local_addr()?)
    }
}

/// Represents an accepted client connection
///
/// Provides methods to send and receive OpenIGTLink messages over the connection.
pub struct IgtlConnection {
    stream: TcpStream,
    verify_crc: bool,
}

impl IgtlConnection {
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
        self.verify_crc = verify;
    }

    /// Get current CRC verification setting
    ///
    /// # Returns
    ///
    /// true if CRC verification is enabled, false otherwise
    pub fn verify_crc(&self) -> bool {
        self.verify_crc
    }

    /// Send a message to the connected client
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
    /// use openigtlink_rust::io::IgtlServer;
    /// use openigtlink_rust::protocol::types::StatusMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let server = IgtlServer::bind("127.0.0.1:18944")?;
    /// let mut conn = server.accept()?;
    ///
    /// let status = StatusMessage::ok("Ready");
    /// let msg = IgtlMessage::new(status, "Server")?;
    /// conn.send(&msg)?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        self.stream.write_all(&data)?;
        self.stream.flush()?;
        Ok(())
    }

    /// Receive a message from the connected client
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
    /// use openigtlink_rust::io::IgtlServer;
    /// use openigtlink_rust::protocol::types::TransformMessage;
    /// use openigtlink_rust::protocol::message::IgtlMessage;
    ///
    /// let server = IgtlServer::bind("127.0.0.1:18944")?;
    /// let mut conn = server.accept()?;
    ///
    /// let msg: IgtlMessage<TransformMessage> = conn.receive()?;
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

        // Decode full message with CRC verification setting
        let mut full_msg = header_buf;
        full_msg.extend_from_slice(&body_buf);

        IgtlMessage::decode_with_options(&full_msg, self.verify_crc)
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

    /// Enable or disable TCP_NODELAY (Nagle's algorithm)
    ///
    /// See [`IgtlClient::set_nodelay`](crate::io::IgtlClient::set_nodelay) for details.
    pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.stream.set_nodelay(nodelay)?;
        Ok(())
    }

    /// Set the size of the TCP receive buffer (SO_RCVBUF)
    ///
    /// See [`IgtlClient::set_recv_buffer_size`](crate::io::IgtlClient::set_recv_buffer_size) for details.
    pub fn set_recv_buffer_size(&self, size: usize) -> Result<()> {
        use std::os::fd::AsRawFd;

        let fd = self.stream.as_raw_fd();
        let size = size as libc::c_int;

        unsafe {
            let ret = libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_RCVBUF,
                &size as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::c_int>() as libc::socklen_t,
            );

            if ret != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
        }

        Ok(())
    }

    /// Set the size of the TCP send buffer (SO_SNDBUF)
    ///
    /// See [`IgtlClient::set_send_buffer_size`](crate::io::IgtlClient::set_send_buffer_size) for details.
    pub fn set_send_buffer_size(&self, size: usize) -> Result<()> {
        use std::os::fd::AsRawFd;

        let fd = self.stream.as_raw_fd();
        let size = size as libc::c_int;

        unsafe {
            let ret = libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_SNDBUF,
                &size as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::c_int>() as libc::socklen_t,
            );

            if ret != 0 {
                return Err(std::io::Error::last_os_error().into());
            }
        }

        Ok(())
    }

    /// Get the current TCP_NODELAY setting
    pub fn nodelay(&self) -> Result<bool> {
        Ok(self.stream.nodelay()?)
    }

    /// Get the remote peer address
    pub fn peer_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.stream.peer_addr()?)
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require multi-threaded setup
    // See examples/ for end-to-end testing
}
