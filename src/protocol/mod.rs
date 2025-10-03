//! OpenIGTLink protocol implementation module
//!
//! This module contains the core protocol structures and message types.

pub mod crc;
pub mod header;
pub mod message;
pub mod types;

// Re-export commonly used types
pub use header::{Header, TypeName, DeviceName};
pub use crc::{calculate_crc, verify_crc};
pub use message::{Message, IgtlMessage};
pub use types::{CapabilityMessage, StatusMessage, TransformMessage};
