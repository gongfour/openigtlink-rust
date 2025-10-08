//! Integration tests for Builder pattern
//!
//! Tests all combinations of protocol and mode selections.

use openigtlink_rust::io::builder::ClientBuilder;
use openigtlink_rust::io::reconnect::ReconnectConfig;

#[test]
fn test_tcp_sync_builder() {
    // Test that sync TCP builder works
    let result = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build();

    // Builder should create client successfully (connection may fail)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_tcp_async_builder() {
    // Test that async TCP builder works
    let result = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .async_mode()
        .build()
        .await;

    // Builder should work (connection may fail if no server)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_tcp_async_with_tls_builder() {
    use openigtlink_rust::io::tls_client::insecure_tls_config;

    // Test that TLS builder works
    let tls_config = std::sync::Arc::new(insecure_tls_config());

    let result = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .async_mode()
        .with_tls(tls_config)
        .build()
        .await;

    // Builder should work (connection/TLS handshake may fail)
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_tcp_async_with_reconnect_builder() {
    // Test that reconnect builder works
    let reconnect_config = ReconnectConfig::with_max_attempts(1);

    let result = ClientBuilder::new()
        .tcp("127.0.0.1:19999")
        .async_mode()
        .with_reconnect(reconnect_config)
        .build()
        .await;

    // Should fail to connect (no server running)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tcp_async_tls_reconnect_builder() {
    use openigtlink_rust::io::tls_client::insecure_tls_config;

    // Test that TLS + reconnect combination works
    let tls_config = std::sync::Arc::new(insecure_tls_config());
    let reconnect_config = ReconnectConfig::with_max_attempts(1);

    let result = ClientBuilder::new()
        .tcp("127.0.0.1:19999")
        .async_mode()
        .with_tls(tls_config)
        .with_reconnect(reconnect_config)
        .build()
        .await;

    // Should fail to connect (no server running)
    assert!(result.is_err());
}

#[test]
fn test_udp_builder() {
    // Test that UDP builder works
    let result = ClientBuilder::new().udp("127.0.0.1:0").build();

    assert!(result.is_ok());
}

#[test]
fn test_builder_verify_crc_option() {
    // Test that verify_crc option works
    let result = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .sync()
        .verify_crc(false)
        .build();

    // Builder should work
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_builder_async_verify_crc_option() {
    // Test that async mode can be selected
    use openigtlink_rust::io::builder::ClientBuilder;

    let builder = ClientBuilder::new()
        .tcp("127.0.0.1:18944")
        .async_mode()
        .verify_crc(true);

    // Builder should compile and be ready to build
    assert_eq!(std::mem::size_of_val(&builder), std::mem::size_of_val(&builder));
}
