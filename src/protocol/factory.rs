//! Message factory for dynamic message type resolution
//!
//! This module provides a factory pattern for creating OpenIGTLink messages
//! based on the message type name in the header, similar to the C++ implementation.

use crate::error::{IgtlError, Result};
use crate::protocol::any_message::AnyMessage;
use crate::protocol::header::Header;
use crate::protocol::message::IgtlMessage;
use crate::protocol::types::*;

/// Message factory for creating messages dynamically based on type name
///
/// This factory allows receiving and decoding messages without knowing the type
/// at compile time. The message type is determined from the header's type_name field.
///
/// # Examples
///
/// ```no_run
/// # use openigtlink_rust::protocol::factory::MessageFactory;
/// # use openigtlink_rust::protocol::header::Header;
/// # fn example(header: Header, body: &[u8]) -> Result<(), openigtlink_rust::error::IgtlError> {
/// let factory = MessageFactory::new();
/// let message = factory.decode_any(&header, body, true)?;
/// println!("Received {} message", message.message_type());
/// # Ok(())
/// # }
/// ```
pub struct MessageFactory;

impl MessageFactory {
    /// Create a new message factory
    pub fn new() -> Self {
        MessageFactory
    }

    /// Decode a message from header and body bytes
    ///
    /// # Arguments
    ///
    /// * `header` - Parsed message header
    /// * `body` - Raw body bytes (may include extended header, content, and metadata)
    /// * `verify_crc` - Whether to verify CRC checksum
    ///
    /// # Returns
    ///
    /// `AnyMessage` containing the decoded message, or `AnyMessage::Unknown` if the
    /// message type is not recognized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use openigtlink_rust::protocol::factory::MessageFactory;
    /// # use openigtlink_rust::protocol::header::Header;
    /// # fn example(header: Header, body: &[u8]) -> Result<(), openigtlink_rust::error::IgtlError> {
    /// let factory = MessageFactory::new();
    /// let message = factory.decode_any(&header, body, true)?;
    ///
    /// match message {
    ///     openigtlink_rust::protocol::AnyMessage::Transform(msg) => {
    ///         println!("Transform from {}", msg.header.device_name.as_str()?);
    ///     }
    ///     openigtlink_rust::protocol::AnyMessage::Unknown { .. } => {
    ///         println!("Unknown message type");
    ///     }
    ///     _ => {}
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode_any(&self, header: &Header, body: &[u8], verify_crc: bool) -> Result<AnyMessage> {
        use crate::protocol::crc::calculate_crc;

        // Verify CRC if requested
        if verify_crc {
            let calculated_crc = calculate_crc(body);
            if calculated_crc != header.crc {
                return Err(IgtlError::CrcMismatch {
                    expected: header.crc,
                    actual: calculated_crc,
                });
            }
        }

        // Reconstruct full message bytes (header + body)
        let mut full_msg = header.encode().to_vec();
        full_msg.extend_from_slice(body);

        // Get type name
        let type_name = header.type_name.as_str()?;

        // Decode based on type name
        match type_name {
            "TRANSFORM" => Ok(AnyMessage::Transform(
                IgtlMessage::<TransformMessage>::decode_with_options(&full_msg, false)?,
            )),
            "STATUS" => Ok(AnyMessage::Status(
                IgtlMessage::<StatusMessage>::decode_with_options(&full_msg, false)?,
            )),
            "CAPABILITY" => Ok(AnyMessage::Capability(
                IgtlMessage::<CapabilityMessage>::decode_with_options(&full_msg, false)?,
            )),
            "IMAGE" => Ok(AnyMessage::Image(
                IgtlMessage::<ImageMessage>::decode_with_options(&full_msg, false)?,
            )),
            "POSITION" => Ok(AnyMessage::Position(
                IgtlMessage::<PositionMessage>::decode_with_options(&full_msg, false)?,
            )),
            "STRING" => Ok(AnyMessage::String(
                IgtlMessage::<StringMessage>::decode_with_options(&full_msg, false)?,
            )),
            "QTDATA" => Ok(AnyMessage::QtData(
                IgtlMessage::<QtDataMessage>::decode_with_options(&full_msg, false)?,
            )),
            "TDATA" => Ok(AnyMessage::TData(
                IgtlMessage::<TDataMessage>::decode_with_options(&full_msg, false)?,
            )),
            "SENSOR" => Ok(AnyMessage::Sensor(
                IgtlMessage::<SensorMessage>::decode_with_options(&full_msg, false)?,
            )),
            "POINT" => Ok(AnyMessage::Point(
                IgtlMessage::<PointMessage>::decode_with_options(&full_msg, false)?,
            )),
            "TRAJECTORY" => Ok(AnyMessage::Trajectory(
                IgtlMessage::<TrajectoryMessage>::decode_with_options(&full_msg, false)?,
            )),
            "NDARRAY" => Ok(AnyMessage::NdArray(
                IgtlMessage::<NdArrayMessage>::decode_with_options(&full_msg, false)?,
            )),
            "BIND" => Ok(AnyMessage::Bind(
                IgtlMessage::<BindMessage>::decode_with_options(&full_msg, false)?,
            )),
            "COLORTABLE" => Ok(AnyMessage::ColorTable(
                IgtlMessage::<ColorTableMessage>::decode_with_options(&full_msg, false)?,
            )),
            "IMGMETA" => Ok(AnyMessage::ImgMeta(
                IgtlMessage::<ImgMetaMessage>::decode_with_options(&full_msg, false)?,
            )),
            "LBMETA" => Ok(AnyMessage::LbMeta(
                IgtlMessage::<LbMetaMessage>::decode_with_options(&full_msg, false)?,
            )),
            "POLYDATA" => Ok(AnyMessage::PolyData(
                IgtlMessage::<PolyDataMessage>::decode_with_options(&full_msg, false)?,
            )),
            "VIDEO" => Ok(AnyMessage::Video(
                IgtlMessage::<VideoMessage>::decode_with_options(&full_msg, false)?,
            )),
            "VIDEOMETA" => Ok(AnyMessage::VideoMeta(
                IgtlMessage::<VideoMetaMessage>::decode_with_options(&full_msg, false)?,
            )),
            "COMMAND" => Ok(AnyMessage::Command(
                IgtlMessage::<CommandMessage>::decode_with_options(&full_msg, false)?,
            )),

            // Query messages
            "GET_TRANS" => Ok(AnyMessage::GetTransform(
                IgtlMessage::<GetTransformMessage>::decode_with_options(&full_msg, false)?,
            )),
            "GET_STATUS" => Ok(AnyMessage::GetStatus(
                IgtlMessage::<GetStatusMessage>::decode_with_options(&full_msg, false)?,
            )),
            "GET_CAPABIL" => Ok(AnyMessage::GetCapability(IgtlMessage::<
                GetCapabilityMessage,
            >::decode_with_options(
                &full_msg, false
            )?)),
            "GET_IMAGE" => Ok(AnyMessage::GetImage(
                IgtlMessage::<GetImageMessage>::decode_with_options(&full_msg, false)?,
            )),
            "GET_IMGMETA" => Ok(AnyMessage::GetImgMeta(
                IgtlMessage::<GetImgMetaMessage>::decode_with_options(&full_msg, false)?,
            )),
            "GET_LBMETA" => Ok(AnyMessage::GetLbMeta(
                IgtlMessage::<GetLbMetaMessage>::decode_with_options(&full_msg, false)?,
            )),
            "GET_POINT" => Ok(AnyMessage::GetPoint(
                IgtlMessage::<GetPointMessage>::decode_with_options(&full_msg, false)?,
            )),
            "GET_TDATA" => Ok(AnyMessage::GetTData(
                IgtlMessage::<GetTDataMessage>::decode_with_options(&full_msg, false)?,
            )),

            // Response messages
            "RTS_TRANS" => Ok(AnyMessage::RtsTransform(
                IgtlMessage::<RtsTransformMessage>::decode_with_options(&full_msg, false)?,
            )),
            "RTS_STATUS" => Ok(AnyMessage::RtsStatus(
                IgtlMessage::<RtsStatusMessage>::decode_with_options(&full_msg, false)?,
            )),
            "RTS_CAPABIL" => Ok(AnyMessage::RtsCapability(IgtlMessage::<
                RtsCapabilityMessage,
            >::decode_with_options(
                &full_msg, false
            )?)),
            "RTS_IMAGE" => Ok(AnyMessage::RtsImage(
                IgtlMessage::<RtsImageMessage>::decode_with_options(&full_msg, false)?,
            )),
            "RTS_TDATA" => Ok(AnyMessage::RtsTData(
                IgtlMessage::<RtsTDataMessage>::decode_with_options(&full_msg, false)?,
            )),

            // Streaming control messages
            "STT_TDATA" => Ok(AnyMessage::StartTData(
                IgtlMessage::<StartTDataMessage>::decode_with_options(&full_msg, false)?,
            )),
            "STP_TRANS" => Ok(AnyMessage::StopTransform(IgtlMessage::<
                StopTransformMessage,
            >::decode_with_options(
                &full_msg, false
            )?)),
            "STP_POSITION" => Ok(AnyMessage::StopPosition(
                IgtlMessage::<StopPositionMessage>::decode_with_options(&full_msg, false)?,
            )),
            "STP_QTDATA" => Ok(AnyMessage::StopQtData(
                IgtlMessage::<StopQtDataMessage>::decode_with_options(&full_msg, false)?,
            )),
            "STP_TDATA" => Ok(AnyMessage::StopTData(
                IgtlMessage::<StopTDataMessage>::decode_with_options(&full_msg, false)?,
            )),
            "STP_IMAGE" => Ok(AnyMessage::StopImage(
                IgtlMessage::<StopImageMessage>::decode_with_options(&full_msg, false)?,
            )),
            "STP_NDARRAY" => Ok(AnyMessage::StopNdArray(
                IgtlMessage::<StopNdArrayMessage>::decode_with_options(&full_msg, false)?,
            )),

            // Unknown message type - store header and body for manual processing
            _ => Ok(AnyMessage::Unknown {
                header: header.clone(),
                body: body.to_vec(),
            }),
        }
    }
}

impl Default for MessageFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::types::TransformMessage;

    #[test]
    fn test_factory_decode_transform() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();
        let encoded = msg.encode().unwrap();

        // Parse header
        let header = Header::decode(&encoded[..Header::SIZE]).unwrap();
        let body = &encoded[Header::SIZE..];

        // Decode with factory
        let factory = MessageFactory::new();
        let any_msg = factory.decode_any(&header, body, true).unwrap();

        assert_eq!(any_msg.message_type(), "TRANSFORM");
        assert!(any_msg.as_transform().is_some());
        assert_eq!(any_msg.device_name().unwrap(), "TestDevice");
    }

    #[test]
    fn test_factory_decode_status() {
        let status = StatusMessage::ok("Test message");
        let msg = IgtlMessage::new(status, "StatusDevice").unwrap();
        let encoded = msg.encode().unwrap();

        let header = Header::decode(&encoded[..Header::SIZE]).unwrap();
        let body = &encoded[Header::SIZE..];

        let factory = MessageFactory::new();
        let any_msg = factory.decode_any(&header, body, true).unwrap();

        assert_eq!(any_msg.message_type(), "STATUS");
        assert!(any_msg.as_status().is_some());
    }

    #[test]
    fn test_factory_unknown_type() {
        // Create a header with unknown type
        use crate::protocol::header::{DeviceName, Timestamp, TypeName};

        let header = Header {
            version: 2,
            type_name: TypeName::new("CUSTOM").unwrap(),
            device_name: DeviceName::new("Device").unwrap(),
            timestamp: Timestamp::now(),
            body_size: 4,
            crc: 0,
        };

        let body = vec![1, 2, 3, 4];

        let factory = MessageFactory::new();
        let any_msg = factory.decode_any(&header, &body, false).unwrap();

        assert!(any_msg.is_unknown());
        assert_eq!(any_msg.message_type(), "CUSTOM");
    }

    #[test]
    fn test_factory_crc_verification() {
        let transform = TransformMessage::identity();
        let msg = IgtlMessage::new(transform, "TestDevice").unwrap();
        let mut encoded = msg.encode().unwrap();

        let header = Header::decode(&encoded[..Header::SIZE]).unwrap();

        // Corrupt the body
        encoded[Header::SIZE] ^= 0xFF;
        let body = &encoded[Header::SIZE..];

        let factory = MessageFactory::new();

        // Should fail with CRC verification
        let result = factory.decode_any(&header, body, true);
        assert!(result.is_err());

        // Should succeed without CRC verification
        let result = factory.decode_any(&header, body, false);
        assert!(result.is_ok());
    }
}
