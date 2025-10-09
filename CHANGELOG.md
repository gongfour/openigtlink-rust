# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2] - 2025-10-09

### Fixed

- **Windows Platform Support**
  - Added conditional compilation for platform-specific socket APIs in `set_recv_buffer_size()` and `set_send_buffer_size()`
  - Defined Windows Winsock constants (SOL_SOCKET, SO_RCVBUF, SO_SNDBUF) that are not available in libc crate on Windows
  - Fixed compilation errors on Windows by using `AsRawSocket` instead of `AsRawFd`
  - Fixed type mismatches in setsockopt calls (c_char vs c_void pointer types)

### Changed

- **CI/CD Configuration**
  - Updated MSRV (Minimum Supported Rust Version) from 1.70.0 to 1.84.0 for Cargo.lock version 4 compatibility

## [0.3.1] - 2025-10-09

### Added

- **CI/CD Infrastructure**
  - GitHub Actions CI workflow with comprehensive testing
    - Multi-platform testing (Ubuntu, macOS, Windows)
    - Multi-version Rust testing (stable, beta, MSRV 1.70.0)
    - Clippy linting with warnings as errors
    - Rustfmt code formatting verification
    - Documentation build verification
    - Security audit with rustsec
    - Benchmark execution
    - Example compilation verification
  - GitHub Actions Release workflow
    - Automated crates.io publishing on version tags
    - Pre-publish validation and dry-run
    - Automatic GitHub Release creation with changelog extraction
    - Post-publish verification testing

## [0.3.0] - 2025-10-09

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

## [0.2.0] - 2025-10-09

### Added

- **Type-State Builder Pattern** for compile-time safe client construction
  - `ClientBuilder` API with fluent interface
  - Type-safe protocol selection (TCP/UDP)
  - Type-safe mode selection (sync/async)
  - Optional feature composition (TLS, reconnect, CRC verification)
  - Compile-time prevention of invalid configurations (e.g., UDP + TLS)
- **Unified Async Client** architecture eliminating variant explosion
  - Single `UnifiedAsyncClient` supporting multiple feature combinations
  - TLS + Reconnect combination support (previously impossible)
  - Cleaner codebase with reduced type complexity
  - Optional transport wrapping (Plain TCP or TLS)
- Comprehensive integration tests for all Builder combinations
- Extensive documentation for Builder pattern and UnifiedAsyncClient architecture

### Changed

- **BREAKING**: Deprecated all legacy client APIs in favor of `ClientBuilder`
  - `IgtlClient::connect()` → `ClientBuilder::new().tcp().sync().build()`
  - `AsyncIgtlClient::connect()` → `ClientBuilder::new().tcp().async_mode().build()`
  - `TlsAsyncClient::connect()` → `ClientBuilder::new().tcp().async_mode().with_tls().build()`
- Updated all examples to use new builder API
- Synchronized documentation with current API implementation
- Clarified UDP+TLS incompatibility in documentation

### Removed

- All deprecated client creation functions (kept for backward compatibility with deprecation warnings)

## [0.1.0] - 2025-10-09

### Added

#### Core Protocol (20/20 Message Types)

- **Tracking & Position Messages**
  - TRANSFORM - 4x4 transformation matrices
  - POSITION - Position + quaternion orientation
  - QTDATA - Quaternion tracking for surgical tools
  - TDATA - Transform tracking data (3x4 matrices)
  - TRAJECTORY - 3D surgical trajectories
  - POINT - Fiducial points for navigation

- **Medical Imaging Messages**
  - IMAGE - 2D/3D medical images (CT/MRI/Ultrasound) with compression
  - VIDEO - Video streaming (H.264, VP9, MJPEG, Raw)
  - IMGMETA - Image metadata (patient info, modality)
  - VIDEOMETA - Video metadata (codec, resolution, bitrate)

- **Data & Sensor Messages**
  - SENSOR - Sensor arrays (force sensors, IMU, etc.)
  - NDARRAY - N-dimensional numerical arrays

- **Visualization Messages**
  - POLYDATA - 3D meshes and polygons
  - LBMETA - Segmentation labels
  - COLORTABLE - Color lookup tables

- **Communication Messages**
  - STRING - Text messages
  - COMMAND - XML commands
  - STATUS - Device status
  - CAPABILITY - Protocol negotiation
  - BIND - Message grouping

- **Query/Control Messages** (22 types)
  - GET_* - Query messages for all data types
  - STT_* - Streaming start messages
  - STP_* - Streaming stop messages
  - RTS_* - Response to streaming messages

#### Networking & I/O

- **Synchronous I/O**
  - Blocking TCP client (`SyncIgtlClient`)
  - Blocking TCP server
  - Simple API for straightforward applications

- **Asynchronous I/O**
  - Tokio-based async client (`AsyncIgtlClient`)
  - Tokio-based async server
  - Multi-client session management (`SessionManager`)
  - High-concurrency support (100+ simultaneous clients)

- **UDP Protocol**
  - Low-latency transport for real-time tracking (120+ Hz)
  - Connectionless communication (`UdpClient`)

- **TLS/SSL Encryption**
  - Secure medical data transfer
  - Certificate-based authentication with rustls
  - Certificate validation and custom CA support
  - Compatible with hospital security requirements

- **Auto-reconnection**
  - Exponential backoff strategy
  - Configurable retry limits and intervals
  - Network resilience for unstable connections
  - Transparent reconnection for async clients

#### Performance & Reliability

- **Message Compression**
  - Deflate/Gzip support for medical images
  - 98-99% compression ratio for CT/MRI data
  - Automatic compression/decompression

- **CRC-64 Validation**
  - Optional integrity checking (CRC64-ECMA)
  - C++ library binary compatibility
  - Selective verification mode

- **Message Queue**
  - Buffering for high-throughput scenarios
  - Backpressure management
  - Configurable queue depth

- **Partial Transfer**
  - Resume capability for large messages
  - Network interruption recovery

- **Advanced TCP Options**
  - TCP_NODELAY for low latency
  - SO_KEEPALIVE for connection monitoring
  - Configurable socket buffer sizes

#### Protocol Features

- OpenIGTLink Protocol Version 3 support
  - Extended header with metadata
  - Nanosecond-precision timestamps
  - Message metadata fields
- Binary compatibility with C++ OpenIGTLink 3.0+
- CRC64-ECMA checksum implementation
- Zero-copy parsing for performance
- Structured error handling with `thiserror`

#### Developer Experience

- **Comprehensive Examples** (27 total)
  - `client.rs`, `server.rs` - Basic client/server communication
  - `image_streaming.rs` - Medical imaging workflows (CT/MRI/Ultrasound)
  - `video_streaming.rs` - Video streaming with multiple codecs
  - `tracking_server.rs` - Surgical tool tracking
  - `udp_tracking.rs` - High-speed UDP tracking (120+ Hz)
  - `point_navigation.rs` - Fiducial point registration
  - `sensor_logger.rs` - Sensor data integration
  - `tls_communication.rs` - TLS secure communication
  - `reconnect.rs` - Auto-reconnection example
  - `session_manager.rs` - Multi-client server
  - `query_streaming.rs` - Query and streaming control
  - `compression.rs` - Image compression
  - `message_queue.rs` - High-throughput buffering
  - And many more...

- **Performance Benchmarks**
  - `throughput.rs` - Message throughput testing (10,000+ msg/sec)
  - `compression.rs` - Compression ratio measurement (98-99%)
  - `serialization.rs` - Serialization performance

- **Structured Logging**
  - Tracing instrumentation throughout I/O stack
  - Debug and production logging levels
  - Integration with `tracing-subscriber`

- **Extensive Documentation**
  - Complete API documentation on docs.rs
  - Usage guides and tutorials in README
  - Architecture documentation for builder pattern
  - Query/streaming control guide
  - 9x documentation expansion for core types

- **Testing**
  - 102+ unit and integration tests
  - C++ OpenIGTLink compatibility validation
  - Cross-platform testing

### Technical Details

- Rust 2021 edition
- MSRV: Rust 1.70+
- Dependencies: tokio (async runtime), rustls (TLS), serde (serialization), bytes (buffer management)
- MIT License
- Published on crates.io
- Documentation on docs.rs
