//! OpenIGTLink protocol implementation module
//!
//! This module contains the core protocol structures and message types.

pub mod any_message;
pub mod crc;
pub mod factory;
pub mod header;
pub mod message;
pub mod types;

// Re-export commonly used types
pub use any_message::AnyMessage;
pub use crc::{calculate_crc, verify_crc};
pub use factory::MessageFactory;
pub use header::{DeviceName, Header, Timestamp, TypeName};
pub use message::{IgtlMessage, Message};
pub use types::{CapabilityMessage, StatusMessage, TransformMessage};
