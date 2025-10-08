//! Common helper functions for OpenIGTLink clients
//!
//! This module provides shared encode/decode logic used across different client implementations
//! to reduce code duplication and improve maintainability.

use crate::error::Result;
use crate::protocol::message::{IgtlMessage, Message};
use tracing::{debug, trace, warn};

/// Encode a message to bytes
///
/// This function encodes an OpenIGTLink message and logs the encoding process.
///
/// # Arguments
/// * `msg` - Reference to the message to encode
///
/// # Returns
/// Encoded message as byte vector
///
/// # Examples
/// ```no_run
/// # use openigtlink_rust::protocol::{IgtlMessage, types::TransformMessage};
/// # use openigtlink_rust::io::common::encode_message;
/// let transform = TransformMessage::identity();
/// let msg = IgtlMessage::new(transform, "Device1").unwrap();
/// let data = encode_message(&msg).unwrap();
/// ```
#[inline]
#[allow(dead_code)]
pub(crate) fn encode_message<T: Message>(msg: &IgtlMessage<T>) -> Result<Vec<u8>> {
    let data = msg.encode()?;
    let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
    let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

    debug!(
        msg_type = msg_type,
        device_name = device_name,
        size = data.len(),
        "Encoding message"
    );

    trace!(
        msg_type = msg_type,
        bytes = data.len(),
        "Message encoded successfully"
    );

    Ok(data)
}

/// Decode a message from bytes
///
/// This function decodes an OpenIGTLink message from a byte slice with optional CRC verification
/// and logs the decoding process.
///
/// # Arguments
/// * `data` - Byte slice containing the full encoded message (header + body)
/// * `verify_crc` - Whether to verify the CRC checksum
///
/// # Returns
/// Decoded message or error
///
/// # Examples
/// ```no_run
/// # use openigtlink_rust::protocol::types::TransformMessage;
/// # use openigtlink_rust::io::common::decode_message;
/// # let data: &[u8] = &[];
/// let msg = decode_message::<TransformMessage>(data, true).unwrap();
/// ```
#[inline]
#[allow(dead_code)]
pub(crate) fn decode_message<T: Message>(data: &[u8], verify_crc: bool) -> Result<IgtlMessage<T>> {
    let result = IgtlMessage::decode_with_options(data, verify_crc);

    match &result {
        Ok(msg) => {
            let msg_type = msg.header.type_name.as_str().unwrap_or("UNKNOWN");
            let device_name = msg.header.device_name.as_str().unwrap_or("UNKNOWN");

            debug!(
                msg_type = msg_type,
                device_name = device_name,
                "Message decoded successfully"
            );

            trace!(
                msg_type = msg_type,
                device_name = device_name,
                body_size = msg.header.body_size,
                "Decoding completed"
            );
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to decode message"
            );
        }
    }

    result
}
