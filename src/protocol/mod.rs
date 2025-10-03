//! OpenIGTLink protocol implementation module
//!
//! This module contains the core protocol structures and message types.

pub mod crc;
pub mod header;

// Re-export commonly used types
pub use header::{Header, TypeName, DeviceName};
pub use crc::{calculate_crc, verify_crc};
