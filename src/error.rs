//! Error types for OpenIGTLink protocol operations

use thiserror::Error;

/// OpenIGTLink protocol error types
#[derive(Error, Debug)]
pub enum IgtlError {
    /// Invalid header format or content
    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    /// CRC checksum mismatch
    #[error("CRC mismatch: expected {expected:#x}, got {actual:#x}")]
    CrcMismatch {
        /// Expected CRC value
        expected: u64,
        /// Actual CRC value
        actual: u64,
    },

    /// Unknown or unsupported message type
    #[error("Unknown message type: {0}")]
    UnknownMessageType(String),

    /// Invalid message size
    #[error("Invalid message size: expected {expected}, got {actual}")]
    InvalidSize {
        /// Expected size in bytes
        expected: usize,
        /// Actual size in bytes
        actual: usize,
    },

    /// I/O error occurred during network communication
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// UTF-8 conversion error
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Invalid timestamp value
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// Message body size exceeds maximum allowed
    #[error("Message body too large: {size} bytes (max: {max})")]
    BodyTooLarge {
        /// Actual body size
        size: usize,
        /// Maximum allowed size
        max: usize,
    },
}

/// Result type alias for OpenIGTLink operations
pub type Result<T> = std::result::Result<T, IgtlError>;
