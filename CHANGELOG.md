# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-01-09

### Added

- **Dynamic message dispatch**: New `receive_any()` method for receiving messages without knowing the type in advance
  - Similar to OpenIGTLink C++ implementation's `MessageFactory::CreateReceiveMessage()`
  - Allows generic receivers, monitoring tools, and protocol debugging
  - Supports all 40+ OpenIGTLink message types

- **`AnyMessage` enum** ([src/protocol/any_message.rs](src/protocol/any_message.rs))
  - Wraps any OpenIGTLink message type (TRANSFORM, STATUS, IMAGE, CAPABILITY, etc.)
  - `Unknown` variant for custom/unsupported message types
  - Convenience methods:
    - `message_type()` - Get message type name as string
    - `device_name()` - Get device name from header
    - `header()` - Get reference to message header
    - `as_transform()`, `as_status()`, `as_image()`, etc. - Type-specific extractors
    - `is_unknown()` - Check if message type is unknown

- **`MessageFactory`** ([src/protocol/factory.rs](src/protocol/factory.rs))
  - Runtime message type resolution based on header's `type_name` field
  - Automatic message decoding and dispatch
  - CRC verification support
  - Handles all standard OpenIGTLink message types

- **Client API additions**:
  - `SyncTcpClient::receive_any()` - Synchronous dynamic message receiving
  - `UnifiedAsyncClient::receive_any()` - Asynchronous dynamic message receiving
  - `SyncIgtlClient::receive_any()` - Unified sync client wrapper
  - `AsyncIgtlClient::receive_any()` - Unified async client wrapper

- **Examples**:
  - `examples/dynamic_receiver.rs` - Synchronous dynamic message receiver
  - `examples/async_dynamic_receiver.rs` - Asynchronous dynamic message receiver with Ctrl+C handling

### Changed

- Re-exported `AnyMessage` and `MessageFactory` from `protocol` module for easier access

### Technical Details

This release achieves feature parity with the OpenIGTLink C++ library's MessageFactory pattern,
allowing Rust applications to receive and handle messages dynamically without compile-time type knowledge.

**Use Case**: Perfect for building generic OpenIGTLink receivers, monitoring tools, protocol
analyzers, or applications that need to handle multiple message types from various devices.

**Example**:
```rust
use openigtlink_rust::io::builder::ClientBuilder;
use openigtlink_rust::protocol::AnyMessage;

let mut client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;

loop {
    let msg = client.receive_any()?;

    match msg {
        AnyMessage::Transform(transform_msg) => {
            println!("Transform from {}", transform_msg.header.device_name.as_str()?);
        }
        AnyMessage::Status(status_msg) => {
            println!("Status: {}", status_msg.content.status_string);
        }
        AnyMessage::Unknown { header, body } => {
            println!("Unknown type: {}", header.type_name.as_str()?);
        }
        _ => {}
    }
}
```

## [0.2.0] - 2025-01-XX

### Changed

- Refactored to use `ClientBuilder` pattern for all client creation
- Removed all deprecated client APIs
- Unified async client to eliminate variant explosion

### Added

- Comprehensive documentation for Builder pattern
- Integration tests for all Builder combinations

## [0.1.0] - Initial Release

- Initial implementation of OpenIGTLink protocol in Rust
- Support for major message types (TRANSFORM, STATUS, IMAGE, etc.)
- TCP/UDP transport support
- TLS encryption support
- Synchronous and asynchronous APIs
