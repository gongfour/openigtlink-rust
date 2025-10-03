//! OpenIGTLink protocol implementation module
//!
//! This module contains the core protocol structures and message types.

pub mod header;

// Re-export commonly used types
pub use header::{Header, TypeName, DeviceName};
