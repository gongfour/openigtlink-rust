//! OpenIGTLink Protocol Implementation in Rust
//!
//! This library provides a Rust implementation of the OpenIGTLink protocol,
//! which is an open network protocol for image-guided therapy environments.
//!
//! # Features
//!
//! - Type-safe message handling
//! - Synchronous and asynchronous I/O support
//! - Full protocol compliance with OpenIGTLink Version 2 and 3
//! - Zero-copy message parsing where possible
//!
//! # Example
//!
//! ```no_run
//! // Example will be added as implementation progresses
//! ```

pub mod error;
pub mod io;
pub mod protocol;

// Re-export commonly used types
pub use error::{IgtlError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Basic smoke test to ensure modules are accessible
    }
}
