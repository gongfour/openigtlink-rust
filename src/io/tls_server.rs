//! TLS-encrypted OpenIGTLink server implementation
//!
//! Provides secure server with TLS/SSL encryption.

use crate::error::{IgtlError, Result};
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::{rustls, TlsAcceptor};
use tracing::{debug, info, trace, warn};

/// TLS-encrypted OpenIGTLink server
///
/// Provides secure server with encryption and optional client authentication.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::TlsIgtlServer;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let server = TlsIgtlServer::bind(
///         "0.0.0.0:18944",
///         "cert.pem",
///         "key.pem"
///     ).await?;
///
///     let connection = server.accept().await?;
///     Ok(())
/// }
/// ```
pub struct TlsIgtlServer {
    listener: TcpListener,
    acceptor: TlsAcceptor,
}

impl TlsIgtlServer {
    /// Bind to a local address with TLS using certificate files
    ///
    /// # Arguments
    ///
    /// * `addr` - Local address to bind (e.g., "0.0.0.0:18944")
    /// * `cert_path` - Path to PEM-encoded certificate file
    /// * `key_path` - Path to PEM-encoded private key file
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Failed to bind or load certificates
    pub async fn bind(addr: &str, cert_path: &str, key_path: &str) -> Result<Self> {
        info!(
            addr = addr,
            cert = cert_path,
            key = key_path,
            "Binding TLS-enabled OpenIGTLink server"
        );

        // Load certificates
        let certs = Self::load_certs(cert_path)?;
        let key = Self::load_key(key_path)?;

        // Create TLS config
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("TLS config error: {}", e),
                ))
            })?;

        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(addr).await?;
        let local_addr = listener.local_addr()?;

        info!(
            local_addr = %local_addr,
            "TLS server listening"
        );

        Ok(TlsIgtlServer { listener, acceptor })
    }

    /// Bind with custom TLS configuration
    ///
    /// Allows advanced configuration like client authentication, cipher suites, etc.
    pub async fn bind_with_config(addr: &str, config: rustls::ServerConfig) -> Result<Self> {
        info!(addr = addr, "Binding TLS server with custom config");

        let acceptor = TlsAcceptor::from(Arc::new(config));
        let listener = TcpListener::bind(addr).await?;

        info!("TLS server listening with custom config");

        Ok(TlsIgtlServer { listener, acceptor })
    }

    /// Accept a new TLS client connection
    ///
    /// Performs TCP accept followed by TLS handshake.
    pub async fn accept(&self) -> Result<TlsIgtlConnection> {
        trace!("Waiting for TLS client connection");

        let (tcp_stream, addr) = self.listener.accept().await?;

        debug!(peer_addr = %addr, "TCP connection accepted, starting TLS handshake");

        let tls_stream = self.acceptor.accept(tcp_stream).await.map_err(|e| {
            warn!(error = %e, peer_addr = %addr, "TLS handshake failed");
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("TLS handshake failed: {}", e),
            ))
        })?;

        info!(peer_addr = %addr, "TLS client connected");

        Ok(TlsIgtlConnection {
            stream: tls_stream,
            verify_crc: true,
        })
    }

    /// Get the local address this server is bound to
    pub fn local_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    // Helper: Load certificates from PEM file
    fn load_certs(path: &str) -> Result<Vec<CertificateDer<'static>>> {
        let file = File::open(path).map_err(|e| {
            IgtlError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to open certificate file {}: {}", path, e),
            ))
        })?;
        let mut reader = BufReader::new(file);

        rustls_pemfile::certs(&mut reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse certificates: {}", e),
                ))
            })
    }

    // Helper: Load private key from PEM file
    fn load_key(path: &str) -> Result<PrivateKeyDer<'static>> {
        let file = File::open(path).map_err(|e| {
            IgtlError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to open key file {}: {}", path, e),
            ))
        })?;
        let mut reader = BufReader::new(file);

        // Try PKCS8 first, then RSA
        rustls_pemfile::private_key(&mut reader)
            .map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse private key: {}", e),
                ))
            })?
            .ok_or_else(|| {
                IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "No private key found in file",
                ))
            })
    }
}

/// TLS-encrypted client connection
pub struct TlsIgtlConnection {
    stream: TlsStream<TcpStream>,
    verify_crc: bool,
}

impl TlsIgtlConnection {
    /// Enable or disable CRC verification
    pub fn set_verify_crc(&mut self, verify: bool) {
        if verify != self.verify_crc {
            info!(verify = verify, "CRC verification setting changed");
            if !verify {
                warn!("CRC verification disabled");
            }
        }
        self.verify_crc = verify;
    }

    /// Get current CRC verification setting
    pub fn verify_crc(&self) -> bool {
        self.verify_crc
    }

    /// Send a message over TLS
    pub async fn send<T: Message>(&mut self, msg: &IgtlMessage<T>) -> Result<()> {
        let data = msg.encode()?;
        let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            size = data.len(),
            "Sending message to TLS client"
        );

        self.stream.write_all(&data).await?;
        self.stream.flush().await?;

        trace!(
            msg_type = msg_type,
            bytes_sent = data.len(),
            "Message sent to TLS client"
        );

        Ok(())
    }

    /// Receive a message over TLS
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        trace!("Waiting for message header from TLS client");

        let mut header_buf = vec![0u8; Header::SIZE];
        self.stream.read_exact(&mut header_buf).await?;

        let header = Header::decode(&header_buf)?;

        let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            body_size = header.body_size,
            "Received message header from TLS client"
        );

        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf).await?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received from TLS client"
        );

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
                    "Failed to decode message from TLS client"
                );
            }
        }

        result
    }

    /// Get the remote peer address
    pub fn peer_addr(&self) -> Result<std::net::SocketAddr> {
        Ok(self.stream.get_ref().0.peer_addr()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full TLS testing requires certificate generation
    // See examples/tls_communication.rs for integration tests

    #[test]
    fn test_module_exists() {
        // Basic test to ensure module compiles
    }
}
