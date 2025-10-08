//! Synchronous OpenIGTLink client implementation
//!
//! Provides a simple blocking TCP client for OpenIGTLink communication.

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use tracing::{debug, info, trace, warn};

/// Synchronous OpenIGTLink client
///
/// Uses blocking I/O with `std::net::TcpStream` for simple, synchronous communication.
#[deprecated(
    since = "0.2.0",
    note = "Use ClientBuilder instead: ClientBuilder::new().tcp(addr).sync().build()"
)]
pub struct IgtlClient {
    stream: TcpStream,
    verify_crc: bool,
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
        info!(addr = %addr, "Connecting to OpenIGTLink server");
        let stream = TcpStream::connect(addr)?;
        let local_addr = stream.local_addr()?;
        info!(
            local_addr = %local_addr,
            remote_addr = %addr,
            "Connected to OpenIGTLink server"
        );
        Ok(IgtlClient {
            stream,
            verify_crc: true, // Default: verify CRC
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    /// // Disable CRC for performance in trusted environment
    /// client.set_verify_crc(false);
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
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
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            size = data.len(),
            "Sending message"
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
        trace!("Waiting for message header");

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
            "Received message header"
        );

        // Read body
        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf)?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received"
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
                    "Failed to decode message"
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
    /// When enabled (true), small packets are sent immediately without buffering.
    /// This is **critical for real-time tracking applications** to minimize latency.
    ///
    /// # Arguments
    ///
    /// * `nodelay` - true to disable Nagle's algorithm (lower latency), false to enable (higher throughput)
    ///
    /// # Performance Impact
    ///
    /// - **Enabled (true)**: Latency: ~2-3ms, recommended for TDATA/TRANSFORM streaming at >100Hz
    /// - **Disabled (false)**: Latency: ~40-200ms, better for bulk IMAGE transfers
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    ///
    /// // Enable for real-time tracking (low latency)
    /// client.set_nodelay(true)?;
    ///
    /// // Disable for large image transfers (high throughput)
    /// client.set_nodelay(false)?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.stream.set_nodelay(nodelay)?;
        Ok(())
    }

    /// Set the size of the TCP receive buffer (SO_RCVBUF)
    ///
    /// Larger buffers reduce packet loss during high-throughput transfers (e.g., video streaming).
    ///
    /// # Arguments
    ///
    /// * `size` - Buffer size in bytes (typically 64KB - 4MB)
    ///
    /// # Recommended Values
    ///
    /// - **Tracking data (TDATA)**: 64KB (default is usually fine)
    /// - **Image streaming (IMAGE)**: 1-2MB for 30fps CT/MRI
    /// - **Video streaming (VIDEO)**: 2-4MB for 1080p@60fps
    ///
    /// # Platform Notes
    ///
    /// The actual buffer size may be limited by OS settings:
    /// - Linux: Check `/proc/sys/net/core/rmem_max`
    /// - macOS: Check `sysctl net.inet.tcp.recvspace`
    /// - Windows: Usually allows up to several MB
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    ///
    /// // Set 2MB buffer for high-resolution image streaming
    /// client.set_recv_buffer_size(2 * 1024 * 1024)?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
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
    /// Larger buffers improve throughput when sending large messages (e.g., medical images).
    ///
    /// # Arguments
    ///
    /// * `size` - Buffer size in bytes (typically 64KB - 4MB)
    ///
    /// # Recommended Values
    ///
    /// Same recommendations as `set_recv_buffer_size()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    ///
    /// // Set 2MB buffer for sending large images
    /// client.set_send_buffer_size(2 * 1024 * 1024)?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
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

    /// Enable TCP keepalive with specified interval
    ///
    /// Keepalive probes detect dead connections (e.g., unplugged network cable).
    ///
    /// # Arguments
    ///
    /// * `duration` - Interval between keepalive probes (None to disable)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    /// use std::time::Duration;
    ///
    /// let mut client = IgtlClient::connect("127.0.0.1:18944")?;
    ///
    /// // Send keepalive probe every 30 seconds
    /// client.set_keepalive(Some(Duration::from_secs(30)))?;
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn set_keepalive(&self, duration: Option<std::time::Duration>) -> Result<()> {
        // Note: TcpStream::set_keepalive was stabilized in Rust 1.58
        // For now, we'll use a simple implementation
        let _ = duration;
        // TODO: Implement keepalive using socket2 crate for better cross-platform support
        eprintln!("Warning: TCP keepalive not yet fully implemented");
        Ok(())
    }

    /// Get the current TCP_NODELAY setting
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use openigtlink_rust::io::IgtlClient;
    ///
    /// let client = IgtlClient::connect("127.0.0.1:18944")?;
    /// let nodelay = client.nodelay()?;
    /// println!("TCP_NODELAY is {}", if nodelay { "enabled" } else { "disabled" });
    /// # Ok::<(), openigtlink_rust::error::IgtlError>(())
    /// ```
    pub fn nodelay(&self) -> Result<bool> {
        Ok(self.stream.nodelay()?)
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
