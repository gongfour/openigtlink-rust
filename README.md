# OpenIGTLink Rust

A Rust implementation of the [OpenIGTLink](http://openigtlink.org/) protocol for image-guided therapy.

[![Crates.io](https://img.shields.io/crates/v/openigtlink-rust)](https://crates.io/crates/openigtlink-rust)
[![Documentation](https://docs.rs/openigtlink-rust/badge.svg)](https://docs.rs/openigtlink-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Overview

OpenIGTLink is an open network protocol for image-guided therapy environments. This library provides a type-safe, performant Rust implementation with 100% compatibility with the official C++ library.

**Project Statistics:**
- 📊 **359 tests** passing
- 📦 **84 source files**
- 📝 **27 examples**
- 🎯 **20/20 message types** implemented
- ✅ **100% C++ compatibility** verified

## Features

### Core Capabilities
- 🦀 **Type-safe**: Leverages Rust's type system for protocol correctness
- 🚀 **Performance**: Zero-copy parsing and efficient serialization
- 🔒 **Memory-safe**: No memory leaks or buffer overflows
- ✅ **Protocol compliance**: Full compatibility with OpenIGTLink Version 2 and 3

### I/O & Networking
- 🔄 **Async/Sync**: Both synchronous and asynchronous I/O with Tokio
- 🌐 **UDP Support**: Connectionless high-speed transmission for low-latency tracking
- 🔐 **TLS/SSL Encryption**: Secure communication with rustls
- 🔁 **Auto-reconnection**: Exponential backoff with jitter
- 📡 **Session Management**: Multi-client server with connection lifecycle

### Advanced Features
- 🗜️ **Compression**: Deflate and Gzip support (98-99% compression for medical images)
- 📦 **Message Queuing**: Bounded/unbounded queues with backpressure control
- ⏸️ **Partial Transfer**: Resume large data transfers after interruption
- 📋 **Structured Logging**: Tracing integration for production debugging
- 🎛️ **CRC Verification**: Optional/configurable CRC-64 validation
- 🏷️ **Extended Headers**: OpenIGTLink v3 metadata and custom headers

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
openigtlink-rust = "0.1.0"
```

## Building from Source

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/gongfour/openigtlink-rust.git
cd openigtlink-rust

# Build the library
cargo build --release

# Run tests (359 tests)
cargo test

# Run benchmarks
cargo bench

# Build documentation
cargo doc --no-deps --open

# Run examples (see Examples section below)
cargo run --example client
```

### Development Build

```bash
# Build in debug mode (faster compilation, slower runtime)
cargo build

# Run with logging
RUST_LOG=debug cargo run --example server

# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Architecture

This library provides multiple client implementations for different use cases:

| Client Type | Use Case | Features |
|-------------|----------|----------|
| `IgtlClient` | Synchronous blocking I/O | Simple, thread-safe |
| `AsyncIgtlClient` | Asynchronous non-blocking | High concurrency with Tokio |
| `TlsIgtlClient` | Encrypted connections | TLS/SSL with certificate validation |
| `ReconnectClient` | Unreliable networks | Automatic reconnection with backoff |
| `UdpClient` | Low-latency tracking | Connectionless high-speed (120+ Hz) |
| `SessionManager` | Multi-client server | Connection lifecycle management |

## Supported Message Types

All 20 OpenIGTLink message types are fully implemented with comprehensive documentation and examples:

### Core Messages
- [x] **TRANSFORM** - Affine transformation matrix (4x4)
- [x] **STATUS** - Device/system status messages
- [x] **CAPABILITY** - Protocol capability negotiation

### Position & Tracking
- [x] **POSITION** - Position + quaternion orientation (compact)
- [x] **QTDATA** - Quaternion tracking data for surgical tools
- [x] **TDATA** - Transform tracking data (3x4 matrices)
- [x] **TRAJECTORY** - 3D trajectory with entry/target points

### Medical Imaging
- [x] **IMAGE** - 2D/3D medical image data with transformations
- [x] **VIDEO** - Real-time video frame streaming (H264/VP9/HEVC/MJPEG/Raw)
- [x] **IMGMETA** - Image metadata (patient info, modality, etc.)
- [x] **VIDEOMETA** - Video stream metadata (codec, resolution, framerate, bitrate)

### Sensors & Data
- [x] **SENSOR** - Sensor data arrays (up to 255 elements)
- [x] **NDARRAY** - N-dimensional numerical arrays

### Navigation & Visualization
- [x] **POINT** - Fiducial points for surgical navigation
- [x] **POLYDATA** - 3D polygon/mesh data for surgical navigation
- [x] **LBMETA** - Label/segmentation metadata
- [x] **COLORTABLE** - Color lookup tables for visualization

### Communication
- [x] **STRING** - Text data transfer with encoding support
- [x] **COMMAND** - XML command messages with ID/name
- [x] **BIND** - Message binding for grouped transmission

### Query & Streaming Control
- [x] **GET_*** - Query messages (GET_CAPABIL, GET_STATUS, GET_TRANSFORM, etc.)
- [x] **STT_*** - Start streaming (STT_TDATA with resolution/coordinate)
- [x] **STP_*** - Stop streaming (STP_TDATA, STP_IMAGE, STP_TRANSFORM, etc.)
- [x] **RTS_*** - Ready-to-send response (RTS_TDATA with status code)

## Protocol Specification

This implementation follows the official OpenIGTLink protocol specification:
- Protocol Version: 2 and 3
- Header Size: 58 bytes
- Byte Order: Big-endian
- CRC: 64-bit (compatible with C++ implementation)

## Examples

This library includes 27 comprehensive examples demonstrating various use cases and features.

### Quick Start

To test the complete client-server communication:

```bash
# Terminal 1: Start the server
cargo run --example server

# Terminal 2: Run the client
cargo run --example client
```

### Basic Examples

**Client-Server Communication**
```bash
# Synchronous client-server
cargo run --example server
cargo run --example client

# Asynchronous communication
cargo run --example async_communication
```

### Medical Imaging

**Image Streaming** - CT/MRI/Ultrasound image transfer
```bash
cargo run --example image_streaming ct          # 512x512x100 CT scan
cargo run --example image_streaming mri         # 256x256x60 MRI scan
cargo run --example image_streaming ultrasound  # 640x480 30fps
```

**Video Streaming** - Real-time video transmission
```bash
cargo run --example video_streaming mjpeg   # MJPEG 640x480 30fps
cargo run --example video_streaming h264    # H.264 1920x1080 60fps
cargo run --example video_streaming raw     # Raw 320x240 15fps
```

### Tracking & Navigation

**Surgical Tool Tracking** - Multi-tool position tracking at 60Hz
```bash
cargo run --example tracking_server
```

**Fiducial Point Registration** - Patient-to-image registration
```bash
cargo run --example point_navigation
```

**Sensor Data Logging** - Force/IMU sensor data collection
```bash
cargo run --example sensor_logger force     # 6-axis force/torque
cargo run --example sensor_logger imu       # 6-axis IMU
cargo run --example sensor_logger combined  # 14 channels
```

### Communication

**Text Commands** - Device control via STRING messages
```bash
cargo run --example string_command
```

**Status Monitoring** - System health and error reporting
```bash
cargo run --example status_monitor
```

**Array Transfer** - N-dimensional numerical arrays
```bash
cargo run --example ndarray_transfer
```

### Advanced Features

**UDP High-Speed Tracking** - Low-latency position updates
```bash
cargo run --example udp_tracking udp            # 120Hz tracking
cargo run --example udp_tracking compare        # TCP vs UDP benchmark
cargo run --example udp_tracking custom 200 5   # Custom FPS/duration
```

**TLS/SSL Encryption** - Secure communication
```bash
# Generate test certificates
./examples/generate_test_certs.sh

# Run TLS client-server demo
cargo run --example tls_communication
```

**Compression** - Reduce bandwidth usage
```bash
cargo run --example compression
# Demonstrates 98-99% compression ratios for medical images
```

**Auto-reconnection** - Network resilience
```bash
cargo run --example reconnect
# Exponential backoff with jitter
```

**Message Queuing** - Backpressure control
```bash
cargo run --example message_queue
# Bounded/unbounded queues with capacity limits
```

**Partial Transfer** - Resume interrupted transfers
```bash
cargo run --example partial_transfer
# Resume large data transfers from checkpoint
```

**Session Management** - Multi-client server
```bash
cargo run --example session_manager
# Handle multiple concurrent clients
```

**Structured Logging** - Production debugging
```bash
RUST_LOG=debug cargo run --example logging
# Tracing integration with filtering
```

**CRC Verification Options** - Performance tuning
```bash
cargo run --example crc_verification_options
# Enable/disable CRC validation per connection
```

**Version 3 Features** - Extended headers and metadata
```bash
cargo run --example version3_extended_header
cargo run --example version3_metadata
```

**Error Handling** - Reconnection, timeout, and recovery patterns
```bash
cargo run --example error_handling reconnect    # Auto-reconnection
cargo run --example error_handling timeout      # Timeout handling
cargo run --example error_handling crc          # CRC error recovery
cargo run --example error_handling wrong_type   # Type mismatch
cargo run --example error_handling all          # All scenarios
```

**Query & Streaming Control** - C++ OpenIGTLink server compatibility
```bash
# Demonstrates GET_CAPABIL, STT_TDATA, and STP_TDATA protocol flow
cargo run --example query_streaming

# Connect to 3D Slicer or PLUS Toolkit server
cargo run --example query_streaming -- 192.168.1.100:18944
```

**Performance Testing** - Benchmarking tools
```bash
cargo run --example performance_test

# Run Criterion benchmarks
cargo bench
```

## Documentation

### API Documentation
- **[docs.rs](https://docs.rs/openigtlink-rust)** - Auto-generated API documentation
- **[Examples](./examples/)** - 27 practical examples with detailed comments
- **[Query & Streaming Guide](./docs/query_usage.md)** - Using query and streaming control messages

### External References
- **[Protocol Specification](https://github.com/openigtlink/OpenIGTLink/blob/master/Documents/Protocol/index.md)** - Official OpenIGTLink protocol
- **[OpenIGTLink Website](http://openigtlink.org/)** - Official website
- **[C++ Library](https://github.com/openigtlink/OpenIGTLink)** - Reference implementation

## Performance

Based on benchmarks and testing:

- **Message throughput**: 10,000+ messages/sec for small messages (TRANSFORM, STATUS)
- **Image encoding**: ~50ms for 512x512x100 CT scan (16-bit)
- **Compression ratio**: 98-99% for typical medical images (zeros, gradients)
- **UDP latency**: <1ms round-trip for TRANSFORM messages
- **Async concurrency**: 100+ simultaneous clients on single thread

Run benchmarks:
```bash
cargo bench --bench throughput
cargo bench --bench compression
cargo bench --bench network
```

## Compatibility

### Tested Against
- ✅ OpenIGTLink C++ 3.0+ (100% binary compatible)
- ✅ 3D Slicer OpenIGTLink extension
- ✅ PLUS Toolkit servers

### Rust Version Support
- **MSRV**: Rust 1.70+
- **Tested on**: 1.70, 1.75, 1.80, stable, nightly

### Platform Support
- ✅ Linux (tested)
- ✅ macOS (tested)
- ✅ Windows (tested)

## Development Status

This project is feature-complete and production-ready:

- ✅ All 20 message types implemented
- ✅ All 22 query/control messages
- ✅ 359 tests passing
- ✅ 100% C++ compatibility
- ✅ Comprehensive documentation
- ✅ Real-world examples

Future work:
- Python bindings
- Protocol extensions

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## References

- [OpenIGTLink Official Website](http://openigtlink.org/)
- [OpenIGTLink C++ Library](https://github.com/openigtlink/OpenIGTLink)
- [Protocol Specification](https://github.com/openigtlink/OpenIGTLink/blob/master/Documents/Protocol/index.md)
- [3D Slicer](https://www.slicer.org/) - Medical image visualization platform
- [PLUS Toolkit](https://plustoolkit.github.io/) - Image-guided intervention toolkit
