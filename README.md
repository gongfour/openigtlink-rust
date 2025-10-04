# OpenIGTLink Rust

A Rust implementation of the [OpenIGTLink](http://openigtlink.org/) protocol for image-guided therapy.

## Overview

OpenIGTLink is an open network protocol for image-guided therapy environments. This library provides a type-safe, performant Rust implementation compatible with the official C++ library.

## Features

- 🦀 **Type-safe**: Leverages Rust's type system for protocol correctness
- 🚀 **Performance**: Zero-copy parsing and efficient serialization
- 🔒 **Memory-safe**: No memory leaks or buffer overflows
- 🔄 **Async/Sync**: Supports both synchronous and asynchronous I/O
- ✅ **Protocol compliance**: Full compatibility with OpenIGTLink Version 2 and 3

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
openigtlink-rust = "0.1.0"
```

## Supported Message Types

- [x] TRANSFORM - Affine transformation matrix (4x4)
- [x] STATUS - Device/system status messages
- [x] CAPABILITY - Protocol capability negotiation
- [x] POSITION - Position + quaternion orientation (compact)
- [x] STRING - Text data transfer with encoding support
- [x] SENSOR - Sensor data arrays (up to 255 elements)
- [x] QTDATA - Quaternion tracking data for surgical tools
- [x] COMMAND - XML command messages with ID/name
- [x] POINT - Fiducial points for surgical navigation
- [x] NDARRAY - N-dimensional numerical arrays
- [x] TDATA - Transform tracking data (3x4 matrices)
- [x] TRAJECTORY - 3D trajectory with entry/target points
- [ ] IMAGE - 2D/3D image data
- [ ] POLYDATA - Polygon/mesh data
- [ ] BIND - Message binding
- [ ] IMGMETA/LBMETA - Image/label metadata
- [ ] COLORTABLE - Color lookup tables
- [ ] VIDEO/VIDEOMETA - Video streaming

## Protocol Specification

This implementation follows the official OpenIGTLink protocol specification:
- Protocol Version: 2 and 3
- Header Size: 58 bytes
- Byte Order: Big-endian
- CRC: 64-bit (compatible with C++ implementation)

## Examples

This library includes example programs to demonstrate client-server communication.

### Running the Server

```bash
# Start server on default port 18944
cargo run --example server

# Start server on custom port
cargo run --example server 12345
```

The server will:
- Accept client connections
- Receive TRANSFORM, STATUS, and CAPABILITY messages
- Send appropriate responses based on message type
- Log all communication to stdout

### Running the Client

```bash
# Test all message types sequentially (default)
cargo run --example client

# Test specific message type
cargo run --example client transform
cargo run --example client status
cargo run --example client capability
```

### Full Test Scenario

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

## Development Status

This project is currently in active development. The basic infrastructure and core message types are being implemented.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## References

- [OpenIGTLink Official Website](http://openigtlink.org/)
- [OpenIGTLink C++ Library](https://github.com/openigtlink/OpenIGTLink)
- [Protocol Specification](https://github.com/openigtlink/OpenIGTLink/blob/master/Documents/Protocol/index.md)
