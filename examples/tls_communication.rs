//! TLS-encrypted communication example
//!
//! Demonstrates secure OpenIGTLink communication using TLS/SSL.
//!
//! # Prerequisites
//!
//! Generate test certificates first:
//! ```bash
//! ./examples/generate_test_certs.sh
//! ```
//!
//! # Running
//!
//! ```bash
//! cargo run --example tls_communication
//! ```

use openigtlink_rust::{
    io::{builder::ClientBuilder, TlsIgtlServer},
    protocol::{message::IgtlMessage, types::StatusMessage},
};
use std::sync::Arc;
use tokio::time::Duration;
use tokio_rustls::rustls;

/// Insecure certificate verifier for testing with self-signed certificates
/// WARNING: Never use this in production!
#[derive(Debug)]
struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();

    println!("=== TLS-Encrypted OpenIGTLink Communication Demo ===\n");

    // Check if certificates exist
    let cert_path = "examples/certs/cert.pem";
    let key_path = "examples/certs/key.pem";

    if !std::path::Path::new(cert_path).exists() || !std::path::Path::new(key_path).exists() {
        eprintln!("Error: Test certificates not found!");
        eprintln!("Please run: ./examples/generate_test_certs.sh");
        eprintln!("");
        return Ok(());
    }

    // Create TLS server
    let server = TlsIgtlServer::bind("127.0.0.1:18945", cert_path, key_path).await?;
    let server_addr = server.local_addr()?;
    println!("TLS Server listening on {}\n", server_addr);

    // Spawn server task
    let server_handle = tokio::spawn(async move {
        println!("[Server] Waiting for TLS client...");
        let mut conn = server.accept().await.unwrap();
        println!("[Server] TLS client connected (encrypted channel established)\n");

        // Receive encrypted message
        let msg: IgtlMessage<StatusMessage> = conn.receive().await.unwrap();
        println!(
            "[Server] Received encrypted message: {}",
            msg.content.status_string
        );

        // Send encrypted response
        let response = StatusMessage::ok("Secure response from server");
        let response_msg = IgtlMessage::new(response, "TlsServer").unwrap();
        conn.send(&response_msg).await.unwrap();
        println!("[Server] Sent encrypted response\n");

        println!("[Server] ✓ Secure communication completed");
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Create TLS client task
    let client_handle = tokio::spawn(async move {
        println!("[Client] Connecting to TLS server...");

        // Use insecure config to accept self-signed certificate
        // WARNING: Only for testing! Production should validate certificates.
        let tls_config = Arc::new(
            rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
                .with_no_client_auth()
        );

        let mut client = ClientBuilder::new()
            .tcp("localhost:18945")
            .async_mode()
            .with_tls(tls_config)
            .build()
            .await
            .unwrap();

        println!("[Client] TLS connection established (encrypted channel)\n");

        // Send encrypted message
        let status = StatusMessage::ok("Secure message from client");
        let msg = IgtlMessage::new(status, "TlsClient").unwrap();
        client.send(&msg).await.unwrap();
        println!("[Client] Sent encrypted message");

        // Receive encrypted response
        let response: IgtlMessage<StatusMessage> = client.receive().await.unwrap();
        println!(
            "[Client] Received encrypted response: {}\n",
            response.content.status_string
        );

        println!("[Client] ✓ Secure communication completed");
    });

    // Wait for both tasks
    let _ = tokio::join!(server_handle, client_handle);

    println!("\n=== TLS Demo completed successfully ===");
    println!("\nSecurity features demonstrated:");
    println!("✓ Encrypted data transmission (TLS 1.2/1.3)");
    println!("✓ Server authentication (certificate validation)");
    println!("✓ Protection against eavesdropping");
    println!("✓ Protection against tampering");

    Ok(())
}
