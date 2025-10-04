//! GET query messages
//!
//! These messages request single data from the server.
//! All GET messages have empty body (body_size = 0).

use super::impl_empty_query;

// GET_CAPABILITY: Query supported message types
impl_empty_query!(GetCapabilityMessage, "GET_CAPABIL");

// GET_STATUS: Query device status
impl_empty_query!(GetStatusMessage, "GET_STATUS");

// GET_TRANSFORM: Query single transform
impl_empty_query!(GetTransformMessage, "GET_TRANSFOR");

// GET_IMAGE: Query single image
impl_empty_query!(GetImageMessage, "GET_IMAGE");

// GET_TDATA: Query tracking data
impl_empty_query!(GetTDataMessage, "GET_TDATA");

// GET_POINT: Query point data
impl_empty_query!(GetPointMessage, "GET_POINT");

// GET_IMGMETA: Query image metadata
impl_empty_query!(GetImgMetaMessage, "GET_IMGMETA");

// GET_LBMETA: Query label metadata
impl_empty_query!(GetLbMetaMessage, "GET_LBMETA");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::message::Message;

    #[test]
    fn test_get_capability_message_type() {
        assert_eq!(GetCapabilityMessage::message_type(), "GET_CAPABIL");
        assert_eq!(GetCapabilityMessage::message_type().len(), 11); // ≤12 chars
    }

    #[test]
    fn test_get_status_empty_body() {
        let msg = GetStatusMessage;
        let encoded = msg.encode_content().unwrap();
        assert_eq!(encoded.len(), 0);
    }

    #[test]
    fn test_get_transform_roundtrip() {
        let original = GetTransformMessage;
        let encoded = original.encode_content().unwrap();
        let decoded = GetTransformMessage::decode_content(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_all_get_messages_type_names() {
        // Verify all type names are ≤12 characters
        assert!(GetCapabilityMessage::message_type().len() <= 12);
        assert!(GetStatusMessage::message_type().len() <= 12);
        assert!(GetTransformMessage::message_type().len() <= 12);
        assert!(GetImageMessage::message_type().len() <= 12);
        assert!(GetTDataMessage::message_type().len() <= 12);
        assert!(GetPointMessage::message_type().len() <= 12);
        assert!(GetImgMetaMessage::message_type().len() <= 12);
        assert!(GetLbMetaMessage::message_type().len() <= 12);
    }
}
