//! Streaming control messages (STT_*, STP_*)
//!
//! - STT_*: Start streaming messages
//! - STP_*: Stop streaming messages

use super::impl_empty_query;

// STP (Stop) messages - all have empty body
impl_empty_query!(StopTDataMessage, "STP_TDATA");
impl_empty_query!(StopImageMessage, "STP_IMAGE");
impl_empty_query!(StopTransformMessage, "STP_TRANSFOR");
impl_empty_query!(StopPositionMessage, "STP_POSITION");
impl_empty_query!(StopQtDataMessage, "STP_QTDATA");
impl_empty_query!(StopNdArrayMessage, "STP_NDARRAY");

// STT (Start) messages will be implemented in the next task
// StartTDataMessage requires parameters (resolution, coordinate_name)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::message::Message;

    #[test]
    fn test_stop_tdata_message_type() {
        assert_eq!(StopTDataMessage::message_type(), "STP_TDATA");
    }

    #[test]
    fn test_stop_image_empty_body() {
        let msg = StopImageMessage;
        let encoded = msg.encode_content().unwrap();
        assert_eq!(encoded.len(), 0);
    }

    #[test]
    fn test_all_stop_messages_type_names() {
        // Verify all type names are ≤12 characters
        assert!(StopTDataMessage::message_type().len() <= 12);
        assert!(StopImageMessage::message_type().len() <= 12);
        assert!(StopTransformMessage::message_type().len() <= 12);
        assert!(StopPositionMessage::message_type().len() <= 12);
        assert!(StopQtDataMessage::message_type().len() <= 12);
        assert!(StopNdArrayMessage::message_type().len() <= 12);
    }
}
