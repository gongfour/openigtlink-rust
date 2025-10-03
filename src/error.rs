//! Error types for OpenIGTLink protocol operations

use thiserror::Error;

/// OpenIGTLink protocol error types
#[derive(Error, Debug)]
pub enum IgtlError {
    /// Placeholder error (will be expanded in next task)
    #[error("Not implemented yet")]
    NotImplemented,
}

/// Result type alias for OpenIGTLink operations
pub type Result<T> = std::result::Result<T, IgtlError>;
