//! OpenIGTLink message type implementations
//!
//! This module contains implementations of various OpenIGTLink message types.

pub mod status;
pub mod transform;

// Re-export message types
pub use status::StatusMessage;
pub use transform::TransformMessage;
