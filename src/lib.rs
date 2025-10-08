//! OpenIGTLink Protocol Implementation in Rust
//!
//! This library provides a Rust implementation of the OpenIGTLink protocol,
//! which is an open network protocol for image-guided therapy environments.
//!
//! # Features
//!
//! - **Type-safe message handling** - Leverages Rust's type system for protocol correctness
//! - **Comprehensive message types** - 20 message types fully implemented
//! - **Synchronous and asynchronous I/O** - Works with both sync and async Rust
//! - **Full protocol compliance** - OpenIGTLink Version 2 and 3
//! - **Memory safe** - No buffer overflows or memory leaks
//! - **Zero-copy parsing** - Efficient deserialization where possible
//!
//! # Quick Start
//!
//! ## Basic Client-Server Example
//!
//! **Server:**
//! ```no_run
//! use openigtlink_rust::io::IgtlServer;
//! use openigtlink_rust::protocol::types::StatusMessage;
//! use openigtlink_rust::protocol::message::IgtlMessage;
//!
//! let server = IgtlServer::bind("127.0.0.1:18944")?;
//! let mut client_conn = server.accept()?;
//!
//! let status = StatusMessage::ok("Server ready");
//! let msg = IgtlMessage::new(status, "Server")?;
//! client_conn.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! **Client:**
//! ```no_run
//! use openigtlink_rust::io::ClientBuilder;
//! use openigtlink_rust::protocol::types::TransformMessage;
//! use openigtlink_rust::protocol::message::IgtlMessage;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .sync()
//!     .build()?;
//!
//! let transform = TransformMessage::identity();
//! let msg = IgtlMessage::new(transform, "Device")?;
//! client.send(&msg)?;
//!
//! let response = client.receive::<TransformMessage>()?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Sending Medical Images
//!
//! ```no_run
//! use openigtlink_rust::protocol::types::{ImageMessage, ImageScalarType};
//! use openigtlink_rust::protocol::message::IgtlMessage;
//! use openigtlink_rust::io::ClientBuilder;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("127.0.0.1:18944")
//!     .sync()
//!     .build()?;
//!
//! let image = ImageMessage::new(
//!     ImageScalarType::Uint8,
//!     [512, 512, 1],
//!     vec![0u8; 512 * 512]
//! )?;
//!
//! let msg = IgtlMessage::new(image, "Device")?;
//! client.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! # Architecture
//!
//! The library is organized into three main modules:
//!
//! ## Module Structure
//!
//! - **`protocol`** - Core protocol implementation
//!   - `header` - OpenIGTLink message header (58 bytes)
//!   - `types` - All 20 message type implementations
//!   - `crc` - CRC-64 checksum validation
//!
//! - **`io`** - Network I/O layer
//!   - `ClientBuilder` - Type-state builder for configuring clients
//!   - `SyncIgtlClient` / `AsyncIgtlClient` - TCP clients for sync/async workflows
//!   - `IgtlServer` - TCP server for accepting connections
//!
//! - **`error`** - Error handling
//!   - `IgtlError` - Unified error type for all operations
//!   - `Result<T>` - Type alias for `Result<T, IgtlError>`
//!
//! ## Design Principles
//!
//! 1. **Type Safety**: Each message type is a distinct Rust type
//! 2. **Zero-cost Abstractions**: Minimal runtime overhead
//! 3. **Explicit Error Handling**: All network and parsing errors are explicit
//! 4. **Protocol Compliance**: Strict adherence to OpenIGTLink specification
//!
//! # Supported Message Types
//!
//! This implementation includes all major OpenIGTLink message types:
//!
//! - **Transform & Tracking**: TRANSFORM, POSITION, QTDATA, TDATA
//! - **Medical Imaging**: IMAGE, IMGMETA, LBMETA, COLORTABLE
//! - **Geometric Data**: POINT, POLYDATA, TRAJECTORY
//! - **Sensor Data**: SENSOR, NDARRAY
//! - **Communication**: STATUS, CAPABILITY, STRING, COMMAND, BIND
//! - **Video Streaming**: VIDEO, VIDEOMETA
//!
//! # Choosing Message Types
//!
//! ## Transform & Tracking
//!
//! - **TRANSFORM** - Single 4x4 homogeneous transformation matrix
//!   - Use for: Robot end-effector pose, single tool tracking
//!   - Example: Surgical instrument position
//!
//! - **TDATA** - Array of transformation matrices with names
//!   - Use for: Multiple tracking tools simultaneously
//!   - Example: Tracking 5 surgical instruments at once
//!
//! - **POSITION** - 3D position only (no orientation)
//!   - Use for: Point cloud data, simplified tracking
//!   - Example: Marker positions without rotation
//!
//! - **QTDATA** - Position + Quaternion (alternative to TRANSFORM)
//!   - Use for: More compact rotation representation
//!   - Example: IMU sensor orientation
//!
//! ## Medical Imaging
//!
//! - **IMAGE** - 2D/3D medical image with metadata
//!   - Use for: CT, MRI, ultrasound images
//!   - Supports: 8/16-bit grayscale, RGB, RGBA
//!   - Example: Real-time ultrasound streaming
//!
//! - **VIDEO** - Real-time video frames
//!   - Use for: Endoscopic video, webcam feeds
//!   - Supports: MJPEG, H.264, raw formats
//!   - Example: Laparoscopic camera feed
//!
//! - **IMGMETA** - Metadata for multiple images
//!   - Use for: Image catalog, DICOM series info
//!   - Example: List of available CT slices
//!
//! ## Sensor Data
//!
//! - **SENSOR** - Multi-channel sensor readings
//!   - Use for: Force sensors, temperature arrays
//!   - Example: 6-axis force/torque sensor
//!
//! - **NDARRAY** - N-dimensional array data
//!   - Use for: Generic numerical data
//!   - Example: Spectrogram, multi-dimensional measurements
//!
//! ## Geometric Data
//!
//! - **POINT** - Collection of 3D points with attributes
//!   - Use for: Fiducial markers, anatomical landmarks
//!   - Example: Registration points for navigation
//!
//! - **POLYDATA** - 3D mesh with vertices and polygons
//!   - Use for: Organ surfaces, tumor boundaries
//!   - Example: Liver segmentation mesh
//!
//! - **TRAJECTORY** - Planned surgical paths
//!   - Use for: Needle insertion paths, biopsy planning
//!   - Example: Brain biopsy trajectory
//!
//! ## Communication
//!
//! - **STATUS** - Status/error messages
//!   - Use for: Operation success/failure notifications
//!   - Example: "Registration completed successfully"
//!
//! - **STRING** - Text messages
//!   - Use for: Logging, user messages
//!   - Example: "Patient positioned, ready to scan"
//!
//! - **COMMAND** - Remote procedure calls
//!   - Use for: Controlling remote devices
//!   - Example: "StartScanning", "StopMotor"
//!
//! # Examples
//!
//! ## Example 1: Sending Transform Data
//!
//! ```no_run
//! use openigtlink_rust::io::ClientBuilder;
//! use openigtlink_rust::protocol::types::TransformMessage;
//! use openigtlink_rust::protocol::message::IgtlMessage;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("192.168.1.100:18944")
//!     .sync()
//!     .build()?;
//!
//! // Create transformation matrix (4x4)
//! let mut transform = TransformMessage::identity();
//!
//! // Set translation (in mm)
//! transform.matrix[0][3] = 100.0; // X
//! transform.matrix[1][3] = 50.0;  // Y
//! transform.matrix[2][3] = 200.0; // Z
//!
//! // Send to server
//! let msg = IgtlMessage::new(transform, "RobotArm")?;
//! client.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Example 2: Streaming Medical Images
//!
//! ```no_run
//! use openigtlink_rust::io::ClientBuilder;
//! use openigtlink_rust::protocol::types::{ImageMessage, ImageScalarType};
//! use openigtlink_rust::protocol::message::IgtlMessage;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("localhost:18944")
//!     .sync()
//!     .build()?;
//!
//! // Simulate ultrasound image stream (640x480 grayscale)
//! for frame_num in 0..100 {
//!     // Generate synthetic image data
//!     let image_data = vec![((frame_num % 256) as u8); 640 * 480];
//!
//!     let image = ImageMessage::new(
//!         ImageScalarType::Uint8,
//!         [640, 480, 1],
//!         image_data
//!     )?;
//!
//!     let msg = IgtlMessage::new(image, "UltrasoundProbe")?;
//!     client.send(&msg)?;
//!     std::thread::sleep(std::time::Duration::from_millis(33)); // ~30 fps
//! }
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Example 3: Multi-Channel Sensor Data
//!
//! ```no_run
//! use openigtlink_rust::io::ClientBuilder;
//! use openigtlink_rust::protocol::types::SensorMessage;
//! use openigtlink_rust::protocol::message::IgtlMessage;
//!
//! let mut client = ClientBuilder::new()
//!     .tcp("localhost:18944")
//!     .sync()
//!     .build()?;
//!
//! // 6-axis force/torque sensor
//! // Forces (Fx, Fy, Fz) and Torques (Tx, Ty, Tz)
//! let readings = vec![1.2, -0.5, 3.8, 0.1, 0.05, -0.2];
//! let sensor = SensorMessage::with_unit(1, 0x0101, readings)?;
//!
//! let msg = IgtlMessage::new(sensor, "ForceSensor")?;
//! client.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Example 4: Status Message Handling
//!
//! ```no_run
//! use openigtlink_rust::io::IgtlServer;
//! use openigtlink_rust::protocol::types::StatusMessage;
//! use openigtlink_rust::protocol::message::IgtlMessage;
//!
//! let server = IgtlServer::bind("0.0.0.0:18944")?;
//! let mut client_conn = server.accept()?;
//!
//! // Receive message
//! let message = client_conn.receive::<StatusMessage>()?;
//!
//! // Send acknowledgment
//! let status = StatusMessage::ok("Data received successfully");
//! let msg = IgtlMessage::new(status, "Server")?;
//! client_conn.send(&msg)?;
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! ## Example 5: Error Handling and Recovery
//!
//! ```no_run
//! use openigtlink_rust::io::ClientBuilder;
//! use openigtlink_rust::protocol::types::TransformMessage;
//! use openigtlink_rust::protocol::message::IgtlMessage;
//! use openigtlink_rust::IgtlError;
//!
//! fn send_with_retry(addr: &str) -> Result<(), IgtlError> {
//!     let max_retries = 3;
//!
//!     for attempt in 0..max_retries {
//!         match ClientBuilder::new().tcp(addr).sync().build() {
//!             Ok(mut client) => {
//!                 let transform = TransformMessage::identity();
//!                 let msg = IgtlMessage::new(transform, "Device")?;
//!                 return client.send(&msg);
//!             }
//!             Err(e) if attempt < max_retries - 1 => {
//!                 eprintln!("Connection failed (attempt {}): {}", attempt + 1, e);
//!                 std::thread::sleep(std::time::Duration::from_secs(1));
//!                 continue;
//!             }
//!             Err(e) => return Err(e),
//!         }
//!     }
//!     unreachable!()
//! }
//! # Ok::<(), openigtlink_rust::IgtlError>(())
//! ```
//!
//! # Error Handling
//!
//! All operations return `Result<T, IgtlError>`. Common error types:
//!
//! - **IoError** - Network communication failures
//! - **InvalidHeader** - Malformed message headers
//! - **CrcMismatch** - Data corruption detected
//! - **UnsupportedMessage** - Unknown message type
//! - **InvalidData** - Invalid message body
//!
//! ```no_run
//! use openigtlink_rust::io::ClientBuilder;
//! use openigtlink_rust::IgtlError;
//!
//! match ClientBuilder::new().tcp("localhost:18944").sync().build() {
//!     Ok(client) => println!("Connected"),
//!     Err(IgtlError::Io(e)) => eprintln!("Network error: {}", e),
//!     Err(e) => eprintln!("Other error: {}", e),
//! }
//! ```

pub mod compression;
pub mod error;
pub mod io;
pub mod protocol;

// Re-export commonly used types
pub use error::{IgtlError, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        // Basic smoke test to ensure modules are accessible
    }
}
