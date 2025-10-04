//! Error types for OpenIGTLink protocol operations
//!
//! This module defines all error types that can occur during OpenIGTLink
//! protocol operations, including network I/O, message parsing, and validation.

use thiserror::Error;

/// OpenIGTLink protocol error types
///
/// All operations in this library return `Result<T, IgtlError>` to provide
/// explicit error handling.
#[derive(Error, Debug)]
pub enum IgtlError {
    /// Invalid header format or content
    ///
    /// This error occurs when:
    /// - Header version field is not 1, 2, or 3
    /// - Message type contains invalid characters (non-ASCII or control characters)
    /// - Device name contains invalid characters
    /// - Header size doesn't match expected 58 bytes
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// let err = IgtlError::InvalidHeader("Version must be 1, 2, or 3".to_string());
    /// ```
    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    /// CRC checksum mismatch
    ///
    /// This error occurs when:
    /// - Network transmission corrupted the message data
    /// - Message was tampered with during transmission
    /// - Sender and receiver use incompatible CRC implementations
    /// - Hardware-level data corruption (rare)
    ///
    /// When this error occurs, the message should be discarded and the sender
    /// should retransmit.
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// let err = IgtlError::CrcMismatch {
    ///     expected: 0x1234567890abcdef,
    ///     actual: 0x1234567890abcdee,
    /// };
    /// ```
    #[error("CRC mismatch: expected {expected:#x}, got {actual:#x}")]
    CrcMismatch {
        /// Expected CRC value calculated from message body
        expected: u64,
        /// Actual CRC value received in message header
        actual: u64,
    },

    /// Unknown or unsupported message type
    ///
    /// This error occurs when:
    /// - Receiving a message type not implemented in this library
    /// - Message type field contains invalid characters
    /// - Sender uses a custom/proprietary message type
    /// - Protocol version mismatch (e.g., OpenIGTLink v4 message on v3 receiver)
    ///
    /// The 21 standard message types are supported. Custom message types
    /// will trigger this error unless explicitly added.
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// let err = IgtlError::UnknownMessageType("CUSTOM_MSG".to_string());
    /// ```
    #[error("Unknown message type: {0}")]
    UnknownMessageType(String),

    /// Invalid message size
    ///
    /// This error occurs when:
    /// - Message body size doesn't match the size declared in header
    /// - Required fields are missing in message body
    /// - Array sizes in message don't match declared counts
    /// - Message is truncated during transmission
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// let err = IgtlError::InvalidSize {
    ///     expected: 100,
    ///     actual: 95,
    /// };
    /// ```
    #[error("Invalid message size: expected {expected}, got {actual}")]
    InvalidSize {
        /// Expected size in bytes based on message format
        expected: usize,
        /// Actual size in bytes received or parsed
        actual: usize,
    },

    /// I/O error occurred during network communication
    ///
    /// This error wraps standard library I/O errors and occurs when:
    /// - TCP connection failed or was refused
    /// - Connection lost during transmission (broken pipe)
    /// - Network timeout occurred
    /// - Socket was closed by peer
    /// - Insufficient permissions to bind to port
    ///
    /// Common scenarios:
    /// - Server not running at specified address
    /// - Firewall blocking the connection
    /// - Network cable unplugged during operation
    /// - Server crashed during communication
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// # use std::io;
    /// let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused");
    /// let err = IgtlError::Io(io_err);
    /// ```
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// UTF-8 conversion error
    ///
    /// This error occurs when:
    /// - Device name or string message contains invalid UTF-8 sequences
    /// - Message was created by non-UTF-8 compliant sender
    /// - Data corruption in text fields
    ///
    /// OpenIGTLink string fields should be UTF-8 encoded. This error
    /// indicates the sender is not following the specification.
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
    /// match String::from_utf8(invalid_bytes) {
    ///     Err(e) => {
    ///         let err = IgtlError::Utf8(e);
    ///     }
    ///     _ => {}
    /// }
    /// ```
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Invalid timestamp value
    ///
    /// This error occurs when:
    /// - Timestamp nanoseconds field exceeds 10^9 (invalid)
    /// - Timestamp seconds field is negative (if checked)
    /// - Timestamp represents a date far in the future (system time issue)
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// let err = IgtlError::InvalidTimestamp("Nanoseconds must be < 1000000000".to_string());
    /// ```
    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(String),

    /// Message body size exceeds maximum allowed
    ///
    /// This error occurs when:
    /// - Attempting to send a message larger than protocol limit (typically 4GB)
    /// - Image data exceeds reasonable memory limits
    /// - Malformed message header declares impossibly large body size
    ///
    /// This protects against memory exhaustion attacks and implementation bugs.
    ///
    /// # Example
    /// ```no_run
    /// # use openigtlink_rust::error::IgtlError;
    /// let err = IgtlError::BodyTooLarge {
    ///     size: 5_000_000_000,
    ///     max: 4_294_967_295,
    /// };
    /// ```
    #[error("Message body too large: {size} bytes (max: {max})")]
    BodyTooLarge {
        /// Actual body size in bytes
        size: usize,
        /// Maximum allowed size in bytes
        max: usize,
    },
}

/// Result type alias for OpenIGTLink operations
pub type Result<T> = std::result::Result<T, IgtlError>;
