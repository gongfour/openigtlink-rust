//! OpenIGTLink Protocol Implementation in Rust
//!
//! This library provides a Rust implementation of the OpenIGTLink protocol,
//! which is an open network protocol for image-guided therapy environments.
//!
//! # Features
//!
//! - **Type-safe message handling** - Leverages Rust's type system for protocol correctness
//! - **Comprehensive message types** - 21 message types fully implemented
//! - **Synchronous and asynchronous I/O** - Works with both sync and async Rust
//! - **Full protocol compliance** - OpenIGTLink Version 2 and 3
//! - **Memory safe** - No buffer overflows or memory leaks
//! - **Zero-copy parsing** - Efficient deserialization where possible
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
//! # Example
//!
//! ```no_run
//! use openigtlink_rust::protocol::types::TransformMessage;
//!
//! // Create a transformation matrix
//! let transform = TransformMessage::identity();
//!
//! // See examples/client.rs and examples/server.rs for complete examples
//! ```

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
