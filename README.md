# OpenIGTLink Rust

A Rust implementation of the [OpenIGTLink](http://openigtlink.org/) protocol for image-guided therapy.

## Overview

OpenIGTLink is an open network protocol for image-guided therapy environments. This library provides a type-safe, performant Rust implementation compatible with the official C++ library.

## Features

- 🦀 **Type-safe**: Leverages Rust's type system for protocol correctness
- 🚀 **Performance**: Zero-copy parsing and efficient serialization
- 🔒 **Memory-safe**: No memory leaks or buffer overflows
- 🔄 **Async/Sync**: Supports both synchronous and asynchronous I/O
- 🌐 **UDP Support**: Connectionless high-speed transmission for low-latency applications
- ✅ **Protocol compliance**: Full compatibility with OpenIGTLink Version 2 and 3

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
openigtlink-rust = "0.1.0"
```

## Supported Message Types

All 20 OpenIGTLink message types are fully implemented with comprehensive documentation and examples:

### Core Messages
- [x] **TRANSFORM** - Affine transformation matrix (4x4)
- [x] **STATUS** - Device/system status messages *(see example)*
- [x] **CAPABILITY** - Protocol capability negotiation

### Position & Tracking
- [x] **POSITION** - Position + quaternion orientation (compact)
- [x] **QTDATA** - Quaternion tracking data for surgical tools
- [x] **TDATA** - Transform tracking data (3x4 matrices) *(see example)*
- [x] **TRAJECTORY** - 3D trajectory with entry/target points

### Medical Imaging
- [x] **IMAGE** - 2D/3D medical image data with transformations *(see example)*
- [x] **VIDEO** - Real-time video frame streaming (H264/VP9/HEVC/MJPEG/Raw) *(see example)*
- [x] **IMGMETA** - Image metadata (patient info, modality, etc.)
- [x] **VIDEOMETA** - Video stream metadata (codec, resolution, framerate, bitrate)

### Sensors & Data
- [x] **SENSOR** - Sensor data arrays (up to 255 elements) *(see example)*
- [x] **NDARRAY** - N-dimensional numerical arrays *(see example)*

### Navigation & Visualization
- [x] **POINT** - Fiducial points for surgical navigation *(see example)*
- [x] **POLYDATA** - 3D polygon/mesh data for surgical navigation
- [x] **LBMETA** - Label/segmentation metadata
- [x] **COLORTABLE** - Color lookup tables for visualization

### Communication
- [x] **STRING** - Text data transfer with encoding support *(see example)*
- [x] **COMMAND** - XML command messages with ID/name
- [x] **BIND** - Message binding for grouped transmission

## Protocol Specification

This implementation follows the official OpenIGTLink protocol specification:
- Protocol Version: 2 and 3
- Header Size: 58 bytes
- Byte Order: Big-endian
- CRC: 64-bit (compatible with C++ implementation)

## Examples

This library includes comprehensive examples demonstrating various use cases and features.

### Basic Examples

**Client-Server Communication**
```bash
# Start basic server
cargo run --example server

# Run basic client tests
cargo run --example client
```

### Medical Imaging

**Image Streaming** - CT/MRI/Ultrasound image transfer
```bash
cargo run --example image_streaming ct      # 512x512x100 CT scan
cargo run --example image_streaming mri     # 256x256x60 MRI scan
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
cargo run --example udp_tracking udp        # 120Hz tracking
cargo run --example udp_tracking compare    # TCP vs UDP benchmark
cargo run --example udp_tracking custom 200 5  # Custom FPS/duration
```

**Async Multi-Client Server** - Tokio-based concurrent server
```bash
cargo run --example async_server
```

**Error Handling** - Reconnection, timeout, and recovery patterns
```bash
cargo run --example error_handling reconnect   # Auto-reconnection
cargo run --example error_handling timeout     # Timeout handling
cargo run --example error_handling crc         # CRC error recovery
cargo run --example error_handling wrong_type  # Type mismatch
cargo run --example error_handling all         # All scenarios
```

### Quick Start

To test the complete client-server communication:

```bash
# Terminal 1: Start the server
cargo run --example server

# Terminal 2: Run the client
cargo run --example client all
```

**Expected output:**

**Server side:**
```
Server listening on 127.0.0.1:18944

[INFO] Client connected: 127.0.0.1:xxxxx
[RECV] TRANSFORM from device 'ClientDevice'
       Matrix (first row): [1.00, 0.00, 0.00, 10.00]
[SEND] STATUS (OK) response

[RECV] STATUS from device 'ClientDevice'
       Code: 1, Name: '', Message: 'Client test message'
[SEND] CAPABILITY response

[RECV] CAPABILITY from device 'ClientDevice'
       Supported types (3):
         1. TRANSFORM
         2. STATUS
         3. CAPABILITY

[INFO] Client session completed, closing connection
```

**Client side:**
```
[INFO] Connected to server

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[TEST] Sending TRANSFORM message...
[SEND] Translation vector: (10.0, 20.0, 30.0)
       Matrix (first row): [1.00, 0.00, 0.00, 10.00]
[RECV] STATUS response:
       Code: 1
       Name: ''
       Message: 'Transform received'
✓ TRANSFORM test completed

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[TEST] Sending STATUS message...
[SEND] Code: 1, Message: 'Client test message'
[RECV] CAPABILITY response:
       Supported types (3):
         1. TRANSFORM
         2. STATUS
         3. CAPABILITY
✓ STATUS test completed

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
[TEST] Sending CAPABILITY message...
[SEND] Supported types (3):
         1. TRANSFORM
         2. STATUS
         3. CAPABILITY
[INFO] CAPABILITY sent, server will close connection
✓ CAPABILITY test completed

[INFO] All tests completed successfully
```

## Documentation

- **[API Documentation](https://docs.rs/openigtlink-rust)** - Auto-generated API docs on docs.rs
- **[Examples](./examples/)** - Practical usage examples with detailed comments
- **[Protocol Specification](https://github.com/openigtlink/OpenIGTLink/blob/master/Documents/Protocol/index.md)** - Official OpenIGTLink protocol

## Development Status

This project is currently in active development. All core message types are implemented with comprehensive documentation and examples.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## References

- [OpenIGTLink Official Website](http://openigtlink.org/)
- [OpenIGTLink C++ Library](https://github.com/openigtlink/OpenIGTLink)
- [Protocol Specification](https://github.com/openigtlink/OpenIGTLink/blob/master/Documents/Protocol/index.md)
