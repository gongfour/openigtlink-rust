//! OpenIGTLink message trait and structures
//!
//! This module defines the common interface that all message types must implement,
//! as well as the generic message wrapper structure.

use crate::error::Result;
use crate::protocol::header::Header;
use std::collections::HashMap;

/// Common interface for all OpenIGTLink message types
///
/// Each message type (TRANSFORM, IMAGE, STATUS, etc.) must implement this trait
/// to provide encoding/decoding functionality.
pub trait Message: Sized {
    /// Returns the message type name (e.g., "TRANSFORM", "IMAGE")
    ///
    /// This must match the OpenIGTLink protocol specification.
    fn message_type() -> &'static str;

    /// Encode message content to bytes
    ///
    /// # Returns
    /// Byte vector containing the encoded message content (without header)
    fn encode_content(&self) -> Result<Vec<u8>>;

    /// Decode message content from bytes
    ///
    /// # Arguments
    /// * `data` - Byte slice containing the message content (without header)
    ///
    /// # Returns
    /// Decoded message or error
    fn decode_content(data: &[u8]) -> Result<Self>;
}

/// Complete OpenIGTLink message structure
///
/// Wraps a specific message type with header, optional extended header,
/// and optional metadata.
///
/// # Type Parameters
/// * `T` - Message type that implements the `Message` trait
#[derive(Debug)]
pub struct IgtlMessage<T: Message> {
    /// Message header (58 bytes)
    pub header: Header,
    /// Extended header (Version 3 feature, optional)
    pub extended_header: Option<Vec<u8>>,
    /// Message content
    pub content: T,
    /// Metadata as key-value pairs (Version 3 feature, optional)
    pub metadata: Option<HashMap<String, String>>,
}

impl<T: Message> IgtlMessage<T> {
    /// Create a new message with the given content and device name
    ///
    /// This is a placeholder for now - full implementation will be done
    /// in the next task when we integrate with CRC.
    ///
    /// # Arguments
    /// * `content` - Message content
    /// * `device_name` - Device name (max 20 characters)
    ///
    /// # Returns
    /// New message with generated header
    #[allow(unused_variables)]
    pub fn new(content: T, device_name: &str) -> Self {
        todo!("Will be implemented in Task 9: IgtlMessage complete implementation")
    }

    /// Encode the complete message to bytes
    ///
    /// This will serialize: header + extended_header + content + metadata
    ///
    /// # Returns
    /// Complete message as byte vector
    pub fn encode(&self) -> Result<Vec<u8>> {
        todo!("Will be implemented in Task 9: IgtlMessage complete implementation")
    }

    /// Decode a complete message from bytes
    ///
    /// # Arguments
    /// * `data` - Byte slice containing the complete message
    ///
    /// # Returns
    /// Decoded message or error
    #[allow(unused_variables)]
    pub fn decode(data: &[u8]) -> Result<Self> {
        todo!("Will be implemented in Task 9: IgtlMessage complete implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock message type for testing
    struct TestMessage {
        data: Vec<u8>,
    }

    impl Message for TestMessage {
        fn message_type() -> &'static str {
            "TEST"
        }

        fn encode_content(&self) -> Result<Vec<u8>> {
            Ok(self.data.clone())
        }

        fn decode_content(data: &[u8]) -> Result<Self> {
            Ok(TestMessage {
                data: data.to_vec(),
            })
        }
    }

    #[test]
    fn test_message_trait() {
        assert_eq!(TestMessage::message_type(), "TEST");
    }

    #[test]
    fn test_message_encode_decode() {
        let original = TestMessage {
            data: vec![1, 2, 3, 4, 5],
        };

        let encoded = original.encode_content().unwrap();
        let decoded = TestMessage::decode_content(&encoded).unwrap();

        assert_eq!(original.data, decoded.data);
    }
}
