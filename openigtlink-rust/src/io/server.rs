//! Synchronous OpenIGTLink server implementation
//!
//! Provides a simple blocking TCP server for OpenIGTLink communication.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use tracing::{debug, info, trace, warn};

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use crate::protocol::AnyMessage;

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
        info!(addr = %addr, "Binding OpenIGTLink server");
        let listener = TcpListener::bind(addr)?;
        let local_addr = listener.local_addr()?;
        info!(
            local_addr = %local_addr,
            "OpenIGTLink server listening"
        );
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
        trace!("Waiting for client connection");
        let (stream, addr) = self.listener.accept()?;
        info!(
            peer_addr = %addr,
            "Client connected"
        );
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
        if verify != self.verify_crc {
            info!(verify = verify, "CRC verification setting changed");
            if !verify {
                warn!("CRC verification disabled - use only in trusted environments");
            }
        }
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
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            size = data.len(),
            "Sending message to client"
        );

        self.stream.write_all(&data)?;
        self.stream.flush()?;

        trace!(
            msg_type = msg_type,
            bytes_sent = data.len(),
            "Message sent successfully"
        );

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
        trace!("Waiting for message header from client");

        // Read header (58 bytes)
        let mut header_buf = vec![0u8; Header::SIZE];
        self.stream.read_exact(&mut header_buf)?;

        let header = Header::decode(&header_buf)?;

        let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            body_size = header.body_size,
            version = header.version,
            "Received message header from client"
        );

        // Read body
        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf)?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received from client"
        );

        // Decode full message with CRC verification setting
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
                    "Failed to decode message from client"
                );
            }
        }

        result
    }

    /// Receive any message type dynamically
    ///
    /// This method receives a message without knowing its type in advance,
    /// returning it as an [`AnyMessage`] enum that can be pattern matched.
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Network read failed
    /// - [`IgtlError::InvalidHeader`](crate::error::IgtlError::InvalidHeader) - Malformed header
    /// - [`IgtlError::CrcMismatch`](crate::error::IgtlError::CrcMismatch) - Data corruption detected
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlServer;
    /// use openigtlink_rust::protocol::AnyMessage;
    ///
    /// let server = IgtlServer::bind("127.0.0.1:18944")?;
    /// let mut conn = server.accept()?;
    ///
    /// let msg = conn.receive_any()?;
    /// match msg {
    ///     AnyMessage::Transform(_) => println!("Received transform"),
    ///     AnyMessage::Status(_) => println!("Received status"),
    ///     _ => println!("Received other message"),
    /// }
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn receive_any(&mut self) -> Result<AnyMessage> {
        trace!("Waiting for any message type from client");

        // Read header (58 bytes)
        let mut header_buf = vec![0u8; Header::SIZE];
        self.stream.read_exact(&mut header_buf)?;

        let header = Header::decode(&header_buf)?;

        let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            body_size = header.body_size,
            version = header.version,
            "Received message header from client"
        );

        // Read body
        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf)?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received from client"
        );

        // Decode full message with CRC verification setting
        let mut full_msg = header_buf;
        full_msg.extend_from_slice(&body_buf);

        let result = AnyMessage::decode_with_options(&full_msg, self.verify_crc);

        match &result {
            Ok(_) => {
                debug!(
                    msg_type = msg_type,
                    device_name = device_name,
                    "Message decoded successfully as AnyMessage"
                );
            }
            Err(e) => {
                warn!(
                    msg_type = msg_type,
                    error = %e,
                    "Failed to decode message from client"
                );
            }
        }

        result
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
    /// Wrapper around [`std::net::TcpStream::set_nodelay`].
    pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.stream.set_nodelay(nodelay)?;
        Ok(())
    }

    /// Set the size of the TCP receive buffer (SO_RCVBUF)
    pub fn set_recv_buffer_size(&self, size: usize) -> Result<()> {
        #[cfg(unix)]
        {
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
        }

        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawSocket;

            // Windows Winsock constants
            const SOL_SOCKET: libc::c_int = 0xffff;
            const SO_RCVBUF: libc::c_int = 0x1002;

            let socket = self.stream.as_raw_socket();
            let size = size as libc::c_int;

            unsafe {
                let ret = libc::setsockopt(
                    socket as libc::SOCKET,
                    SOL_SOCKET,
                    SO_RCVBUF,
                    &size as *const _ as *const libc::c_char,
                    std::mem::size_of::<libc::c_int>() as libc::c_int,
                );

                if ret != 0 {
                    return Err(std::io::Error::last_os_error().into());
                }
            }
        }

        Ok(())
    }

    /// Set the size of the TCP send buffer (SO_SNDBUF)
    pub fn set_send_buffer_size(&self, size: usize) -> Result<()> {
        #[cfg(unix)]
        {
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
        }

        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawSocket;

            // Windows Winsock constants
            const SOL_SOCKET: libc::c_int = 0xffff;
            const SO_SNDBUF: libc::c_int = 0x1001;

            let socket = self.stream.as_raw_socket();
            let size = size as libc::c_int;

            unsafe {
                let ret = libc::setsockopt(
                    socket as libc::SOCKET,
                    SOL_SOCKET,
                    SO_SNDBUF,
                    &size as *const _ as *const libc::c_char,
                    std::mem::size_of::<libc::c_int>() as libc::c_int,
                );

                if ret != 0 {
                    return Err(std::io::Error::last_os_error().into());
                }
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
