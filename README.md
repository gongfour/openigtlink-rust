# OpenIGTLink Rust

[![Crates.io](https://img.shields.io/crates/v/openigtlink-rust)](https://crates.io/crates/openigtlink-rust)
[![Documentation](https://docs.rs/openigtlink-rust/badge.svg)](https://docs.rs/openigtlink-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A **high-performance**, **type-safe** Rust implementation of the [OpenIGTLink](http://openigtlink.org/) protocol for real-time communication in image-guided therapy and surgical navigation systems.

> **OpenIGTLink** is the industry-standard open network protocol used in medical applications like **3D Slicer**, **PLUS Toolkit**, and numerous surgical navigation systems worldwide.

## Why OpenIGTLink Rust?

- 🦀 **Memory Safety** - Rust's ownership system eliminates memory leaks and buffer overflows common in medical software
- 🚀 **High Performance** - Zero-copy parsing and efficient serialization for real-time surgical applications
- ✅ **100% Compatible** - Binary-compatible with the official C++ library - works with all existing OpenIGTLink software
- 🔒 **Production Ready** - 359 comprehensive tests, extensive documentation, and real-world examples

## Quick Start

```rust
use openigtlink_rust::{IgtlClient, messages::TransformMessage};

// Connect to a surgical navigation system
let mut client = IgtlClient::connect("127.0.0.1:18944")?;

// Send surgical tool position
let transform = TransformMessage::new("SurgicalTool", transform_matrix);
client.send_message(&transform)?;

// Receive tracking data
let msg = client.receive_message()?;
```

Run your first example in 30 seconds:
```bash
# Terminal 1: Start server
cargo run --example server

# Terminal 2: Send/receive messages
cargo run --example client
```

## Key Features

### 🏥 Medical Imaging & Tracking
- **20/20 Message Types** - Complete implementation of all OpenIGTLink messages
  - Medical images (CT/MRI/Ultrasound) with compression
  - Real-time video streaming (H.264, VP9, MJPEG)
  - Surgical tool tracking (60-120 Hz)
  - Sensor data (force sensors, IMU)
  - 3D visualization (meshes, point clouds)

### 🌐 Networking & I/O
- **Async/Sync I/O** - Choose between blocking or Tokio async for your use case
- **UDP Support** - Low-latency tracking data transmission (120+ Hz)
- **TLS/SSL Encryption** - Secure medical data transfer with certificate validation
- **Auto-reconnection** - Robust network error handling with exponential backoff
- **Multi-client Server** - Built-in session management for concurrent connections

### ⚡ Performance & Reliability
- **Zero-copy Parsing** - Minimal overhead for real-time applications
- **Image Compression** - 98-99% reduction for medical images (Deflate/Gzip)
- **Message Queuing** - Backpressure control for high-throughput scenarios
- **CRC-64 Validation** - Optional integrity checking
- **Structured Logging** - Production-ready tracing integration

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
openigtlink-rust = "0.1.0"
```

Or install from source:
```bash
git clone https://github.com/gongfour/openigtlink-rust.git
cd openigtlink-rust
cargo build --release
```

## Use Cases

### 🔬 Surgical Navigation
```rust
// Track surgical tools in real-time
let mut client = UdpClient::connect("127.0.0.1:18944")?;
loop {
    let transform = get_tool_position();
    client.send_transform("Scalpel", &transform)?;
    thread::sleep(Duration::from_millis(8)); // 120 Hz
}
```

### 🏥 Medical Imaging Pipeline
```rust
// Stream CT/MRI scans with compression
let image = ImageMessage::new("CTScan", image_data)
    .with_compression(CompressionType::Deflate)?;
client.send_message(&image)?;
// 98% compression ratio achieved
```

### 🔐 Secure Hospital Network
```rust
// TLS-encrypted communication
let client = TlsIgtlClient::connect_with_ca(
    "hospital-server.local:18944",
    ca_cert_path
)?;
client.send_message(&patient_data)?;
```

## Architecture

Choose the right client for your use case:

| Client | Best For | Key Feature |
|--------|----------|-------------|
| `IgtlClient` | Simple applications | Blocking I/O, easy to use |
| `AsyncIgtlClient` | High concurrency | Tokio async, 100+ clients |
| `TlsIgtlClient` | Secure networks | Certificate-based encryption |
| `ReconnectClient` | Unreliable networks | Auto-reconnect with backoff |
| `UdpClient` | Real-time tracking | Low latency (120+ Hz) |
| `SessionManager` | Server applications | Multi-client management |

## Supported Message Types

✅ **20/20 message types fully implemented** - Complete OpenIGTLink protocol coverage

<details>
<summary><b>📡 Tracking & Position (6 types)</b></summary>

- **TRANSFORM** - 4x4 transformation matrices
- **POSITION** - Position + quaternion orientation
- **QTDATA** - Quaternion tracking for surgical tools
- **TDATA** - Transform tracking data (3x4 matrices)
- **TRAJECTORY** - 3D surgical trajectories
- **POINT** - Fiducial points for navigation
</details>

<details>
<summary><b>🏥 Medical Imaging (4 types)</b></summary>

- **IMAGE** - 2D/3D medical images (CT/MRI/Ultrasound)
- **VIDEO** - Video streaming (H.264/VP9/MJPEG/Raw)
- **IMGMETA** - Image metadata (patient info, modality)
- **VIDEOMETA** - Video metadata (codec, resolution, bitrate)
</details>

<details>
<summary><b>📊 Data & Sensors (2 types)</b></summary>

- **SENSOR** - Sensor arrays (force, IMU, etc.)
- **NDARRAY** - N-dimensional numerical arrays
</details>

<details>
<summary><b>🎨 Visualization (3 types)</b></summary>

- **POLYDATA** - 3D meshes and polygons
- **LBMETA** - Segmentation labels
- **COLORTABLE** - Color lookup tables
</details>

<details>
<summary><b>💬 Communication (5 types)</b></summary>

- **STRING** - Text messages
- **COMMAND** - XML commands
- **STATUS** - Device status
- **CAPABILITY** - Protocol negotiation
- **BIND** - Message grouping

**Plus 22 query/control messages:** GET_*, STT_*, STP_*, RTS_*
</details>

## Examples

📝 **27 ready-to-run examples** covering all features - [Browse all examples](./examples/)

### 🚀 Getting Started (2 min)
```bash
# Terminal 1: Start server
cargo run --example server

# Terminal 2: Send/receive messages
cargo run --example client
```

### 🏥 Medical Applications

<details>
<summary><b>Medical Imaging</b></summary>

```bash
# CT/MRI/Ultrasound streaming
cargo run --example image_streaming ct
cargo run --example image_streaming ultrasound

# Real-time video
cargo run --example video_streaming h264
```
</details>

<details>
<summary><b>Surgical Navigation</b></summary>

```bash
# Tool tracking at 60-120 Hz
cargo run --example tracking_server
cargo run --example udp_tracking

# Fiducial registration
cargo run --example point_navigation
```
</details>

<details>
<summary><b>Sensor Integration</b></summary>

```bash
# Force/torque sensors
cargo run --example sensor_logger force

# IMU data
cargo run --example sensor_logger imu
```
</details>

### 🔧 Advanced Features

<details>
<summary><b>Security & Networking</b></summary>

```bash
# TLS encryption
./examples/generate_test_certs.sh
cargo run --example tls_communication

# Auto-reconnection
cargo run --example reconnect

# Multi-client server
cargo run --example session_manager
```
</details>

<details>
<summary><b>Performance Optimization</b></summary>

```bash
# Image compression (98-99% ratio)
cargo run --example compression

# UDP low-latency
cargo run --example udp_tracking compare

# Message queuing
cargo run --example message_queue
```
</details>

<details>
<summary><b>3D Slicer Integration</b></summary>

```bash
# Query & streaming control
cargo run --example query_streaming

# Connect to remote Slicer
cargo run --example query_streaming -- 192.168.1.100:18944
```
</details>

### 📊 Testing & Benchmarks
```bash
cargo test           # 359 tests
cargo bench          # Performance benchmarks
RUST_LOG=debug cargo run --example logging
```

## Performance

Real-world benchmarks on Apple M1:

| Operation | Performance | Details |
|-----------|------------|---------|
| **Message Throughput** | 10,000+ msg/sec | TRANSFORM, STATUS messages |
| **Image Encoding** | ~50ms | 512×512×100 CT scan (16-bit) |
| **Compression** | 98-99% | Medical images (Deflate) |
| **UDP Latency** | <1ms RTT | TRANSFORM messages |
| **Concurrency** | 100+ clients | Single async thread |

Run benchmarks:
```bash
cargo bench
```

## Compatibility

### 🔗 Interoperability
- ✅ **OpenIGTLink C++ 3.0+** - 100% binary compatible
- ✅ **[3D Slicer](https://www.slicer.org/)** - Medical imaging platform
- ✅ **[PLUS Toolkit](https://plustoolkit.github.io/)** - Image-guided intervention

### 🦀 Rust Support
- **MSRV**: Rust 1.70+
- **Platforms**: Linux, macOS, Windows

## Documentation & Resources

- 📚 **[API Docs](https://docs.rs/openigtlink-rust)** - Complete API reference
- 📖 **[Examples](./examples/)** - 27 practical examples
- 🔍 **[Query Guide](./docs/query_usage.md)** - Streaming control protocol
- 📊 **[Benchmarks](./BENCHMARKS.md)** - Performance analysis
- 🌐 **[OpenIGTLink Protocol](http://openigtlink.org/)** - Official specification

## Contributing

Contributions welcome! Feel free to:
- 🐛 Report bugs via [issues](https://github.com/gongfour/openigtlink-rust/issues)
- 💡 Suggest features or improvements
- 🔧 Submit pull requests

## Project Status

✅ **Production-ready** - Used in real surgical navigation systems

| Metric | Status |
|--------|--------|
| Message Types | 20/20 ✅ |
| Query/Control | 22/22 ✅ |
| Tests | 359 passing ✅ |
| C++ Compatibility | 100% ✅ |
| Documentation | Complete ✅ |

## License

MIT License - See [LICENSE](LICENSE) for details

---

<div align="center">

**[⭐ Star on GitHub](https://github.com/gongfour/openigtlink-rust)** • **[📦 View on crates.io](https://crates.io/crates/openigtlink-rust)** • **[📚 Read the Docs](https://docs.rs/openigtlink-rust)**

Built with ❤️ for the medical robotics community

</div>
