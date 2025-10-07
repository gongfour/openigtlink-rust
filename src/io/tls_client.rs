//! TLS-encrypted OpenIGTLink client implementation
//!
//! Provides secure, encrypted communication using TLS/SSL.

use crate::error::{IgtlError, Result};
use crate::protocol::header::Header;
use crate::protocol::message::{IgtlMessage, Message};
use rustls::pki_types::ServerName;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::{rustls, TlsConnector};
use tracing::{debug, info, trace, warn};

/// TLS-encrypted OpenIGTLink client
///
/// Provides secure communication with encryption and authentication.
///
/// # Examples
///
/// ```no_run
/// use openigtlink_rust::io::TlsIgtlClient;
/// use openigtlink_rust::protocol::types::StatusMessage;
/// use openigtlink_rust::protocol::message::IgtlMessage;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Connect with default TLS config (validates server certificates)
///     let mut client = TlsIgtlClient::connect("secure-server.local", 18944).await?;
///
///     let status = StatusMessage::ok("Secure Hello");
///     let msg = IgtlMessage::new(status, "TlsClient")?;
///     client.send(&msg).await?;
///
///     Ok(())
/// }
/// ```
pub struct TlsIgtlClient {
    stream: TlsStream<TcpStream>,
    verify_crc: bool,
}

impl TlsIgtlClient {
    /// Connect to a TLS-enabled OpenIGTLink server
    ///
    /// Uses system root certificates for server validation.
    ///
    /// # Arguments
    ///
    /// * `hostname` - Server hostname (for SNI and certificate validation)
    /// * `port` - Server port
    ///
    /// # Errors
    ///
    /// - [`IgtlError::Io`](crate::error::IgtlError::Io) - Connection or TLS handshake failed
    pub async fn connect(hostname: &str, port: u16) -> Result<Self> {
        info!(
            hostname = hostname,
            port = port,
            "Connecting to TLS-enabled OpenIGTLink server"
        );

        // Create TLS config with system root certificates
        let mut root_store = rustls::RootCertStore::empty();
        let native_certs = rustls_native_certs::load_native_certs();

        for cert in native_certs.certs {
            root_store.add(cert).map_err(|e| {
                IgtlError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to add root certificate: {}", e),
                ))
            })?;
        }

        // Log any errors from loading native certs
        if !native_certs.errors.is_empty() {
            warn!(
                error_count = native_certs.errors.len(),
                "Some native certificates failed to load"
            );
        }

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));

        // Connect TCP
        let addr = format!("{}:{}", hostname, port);
        let tcp_stream = TcpStream::connect(&addr).await?;
        let local_addr = tcp_stream.local_addr()?;

        // Perform TLS handshake
        let server_name = ServerName::try_from(hostname.to_string()).map_err(|e| {
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid hostname: {}", e),
            ))
        })?;

        let stream = connector.connect(server_name, tcp_stream).await.map_err(|e| {
            warn!(error = %e, "TLS handshake failed");
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!("TLS handshake failed: {}", e),
            ))
        })?;

        info!(
            local_addr = %local_addr,
            remote_addr = %addr,
            "TLS connection established"
        );

        Ok(TlsIgtlClient {
            stream,
            verify_crc: true,
        })
    }

    /// Connect with custom TLS configuration
    ///
    /// Allows using custom certificates, disabling verification, etc.
    ///
    /// # Arguments
    ///
    /// * `hostname` - Server hostname
    /// * `port` - Server port
    /// * `config` - Custom TLS client configuration
    pub async fn connect_with_config(
        hostname: &str,
        port: u16,
        config: rustls::ClientConfig,
    ) -> Result<Self> {
        info!(
            hostname = hostname,
            port = port,
            "Connecting to TLS server with custom config"
        );

        let connector = TlsConnector::from(Arc::new(config));
        let addr = format!("{}:{}", hostname, port);
        let tcp_stream = TcpStream::connect(&addr).await?;

        let server_name = ServerName::try_from(hostname.to_string()).map_err(|e| {
            IgtlError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid hostname: {}", e),
            ))
        })?;

        let stream = connector.connect(server_name, tcp_stream).await?;

        info!("TLS connection established with custom config");

        Ok(TlsIgtlClient {
            stream,
            verify_crc: true,
        })
    }

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
            "Sending message over TLS"
        );

        self.stream.write_all(&data).await?;
        self.stream.flush().await?;

        trace!(
            msg_type = msg_type,
            bytes_sent = data.len(),
            "Message sent over TLS"
        );

        Ok(())
    }

    /// Receive a message over TLS
    pub async fn receive<T: Message>(&mut self) -> Result<IgtlMessage<T>> {
        trace!("Waiting for message header over TLS");

        let mut header_buf = vec![0u8; Header::SIZE];
        self.stream.read_exact(&mut header_buf).await?;

        let header = Header::decode(&header_buf)?;

        let msg_type = header.type_name.as_str().unwrap_or("UNKNOWN");
        let device_name = header.device_name.as_str().unwrap_or("UNKNOWN");

        debug!(
            msg_type = msg_type,
            device_name = device_name,
            body_size = header.body_size,
            "Received message header over TLS"
        );

        let mut body_buf = vec![0u8; header.body_size as usize];
        self.stream.read_exact(&mut body_buf).await?;

        trace!(
            msg_type = msg_type,
            bytes_read = body_buf.len(),
            "Message body received over TLS"
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
                    "Failed to decode message"
                );
            }
        }

        result
    }
}

/// Create a TLS client configuration that accepts any certificate (INSECURE)
///
/// **WARNING**: This disables certificate validation and should only be used
/// for testing or in controlled environments. Production systems should use
/// proper certificate validation.
pub fn insecure_tls_config() -> rustls::ClientConfig {
    use rustls::client::danger::{ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::UnixTime;

    #[derive(Debug)]
    struct NoVerifier;

    impl ServerCertVerifier for NoVerifier {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::pki_types::CertificateDer,
            _intermediates: &[rustls::pki_types::CertificateDer],
            _server_name: &rustls::pki_types::ServerName,
            _ocsp_response: &[u8],
            _now: UnixTime,
        ) -> std::result::Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer,
            _dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
        {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _message: &[u8],
            _cert: &rustls::pki_types::CertificateDer,
            _dss: &rustls::DigitallySignedStruct,
        ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error>
        {
            Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
            use rustls::SignatureScheme;
            vec![
                SignatureScheme::RSA_PKCS1_SHA256,
                SignatureScheme::RSA_PKCS1_SHA384,
                SignatureScheme::RSA_PKCS1_SHA512,
                SignatureScheme::ECDSA_NISTP256_SHA256,
                SignatureScheme::ECDSA_NISTP384_SHA384,
                SignatureScheme::ED25519,
                SignatureScheme::RSA_PSS_SHA256,
                SignatureScheme::RSA_PSS_SHA384,
                SignatureScheme::RSA_PSS_SHA512,
            ]
        }
    }

    rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insecure_config_creation() {
        // Should create config without panic
        let _config = insecure_tls_config();
    }
}
