//! Dynamic message dispatching for OpenIGTLink
//!
//! This module provides the `AnyMessage` enum which can hold any message type,
//! allowing for runtime message type detection and handling.

use crate::error::Result;
use crate::protocol::header::Header;
use crate::protocol::message::IgtlMessage;
use crate::protocol::types::*;

/// Enum holding any OpenIGTLink message type
///
/// This allows receiving messages without knowing the type at compile time.
/// The message type is determined at runtime from the header's type_name field.
///
/// # Examples
///
/// ```no_run
/// # use openigtlink_rust::io::builder::ClientBuilder;
/// # use openigtlink_rust::protocol::AnyMessage;
/// # fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
/// let mut client = ClientBuilder::new().tcp("127.0.0.1:18944").sync().build()?;
///
/// let msg = client.receive_any()?;
/// match msg {
///     AnyMessage::Transform(transform_msg) => {
///         println!("Received transform from {}", transform_msg.header.device_name.as_str()?);
///     }
///     AnyMessage::Status(status_msg) => {
///         println!("Status: {}", status_msg.content.status_string);
///     }
///     _ => println!("Other message type"),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub enum AnyMessage {
    /// TRANSFORM message
    Transform(IgtlMessage<TransformMessage>),
    /// STATUS message
    Status(IgtlMessage<StatusMessage>),
    /// CAPABILITY message
    Capability(IgtlMessage<CapabilityMessage>),
    /// IMAGE message
    Image(IgtlMessage<ImageMessage>),
    /// POSITION message
    Position(IgtlMessage<PositionMessage>),
    /// STRING message
    String(IgtlMessage<StringMessage>),
    /// QTDATA message (quaternion tracking data)
    QtData(IgtlMessage<QtDataMessage>),
    /// TDATA message (tracking data)
    TData(IgtlMessage<TDataMessage>),
    /// SENSOR message
    Sensor(IgtlMessage<SensorMessage>),
    /// POINT message
    Point(IgtlMessage<PointMessage>),
    /// TRAJECTORY message
    Trajectory(IgtlMessage<TrajectoryMessage>),
    /// NDARRAY message (n-dimensional array)
    NdArray(IgtlMessage<NdArrayMessage>),
    /// BIND message
    Bind(IgtlMessage<BindMessage>),
    /// COLORTABLE message
    ColorTable(IgtlMessage<ColorTableMessage>),
    /// IMGMETA message (image metadata)
    ImgMeta(IgtlMessage<ImgMetaMessage>),
    /// LBMETA message (label metadata)
    LbMeta(IgtlMessage<LbMetaMessage>),
    /// POLYDATA message
    PolyData(IgtlMessage<PolyDataMessage>),
    /// VIDEO message
    Video(IgtlMessage<VideoMessage>),
    /// VIDEOMETA message
    VideoMeta(IgtlMessage<VideoMetaMessage>),
    /// COMMAND message
    Command(IgtlMessage<CommandMessage>),

    // Query messages (GET_*)
    /// GET_TRANSFORM query message
    GetTransform(IgtlMessage<GetTransformMessage>),
    /// GET_STATUS query message
    GetStatus(IgtlMessage<GetStatusMessage>),
    /// GET_CAPABILITY query message
    GetCapability(IgtlMessage<GetCapabilityMessage>),
    /// GET_IMAGE query message
    GetImage(IgtlMessage<GetImageMessage>),
    /// GET_IMGMETA query message
    GetImgMeta(IgtlMessage<GetImgMetaMessage>),
    /// GET_LBMETA query message
    GetLbMeta(IgtlMessage<GetLbMetaMessage>),
    /// GET_POINT query message
    GetPoint(IgtlMessage<GetPointMessage>),
    /// GET_TDATA query message
    GetTData(IgtlMessage<GetTDataMessage>),

    // Response messages (RTS_*)
    /// RTS_TRANSFORM response message
    RtsTransform(IgtlMessage<RtsTransformMessage>),
    /// RTS_STATUS response message
    RtsStatus(IgtlMessage<RtsStatusMessage>),
    /// RTS_CAPABILITY response message
    RtsCapability(IgtlMessage<RtsCapabilityMessage>),
    /// RTS_IMAGE response message
    RtsImage(IgtlMessage<RtsImageMessage>),
    /// RTS_TDATA response message
    RtsTData(IgtlMessage<RtsTDataMessage>),

    // Streaming control messages (STT_*, STP_*)
    /// STT_TDATA start streaming message
    StartTData(IgtlMessage<StartTDataMessage>),
    /// STP_TRANSFORM stop streaming message
    StopTransform(IgtlMessage<StopTransformMessage>),
    /// STP_POSITION stop streaming message
    StopPosition(IgtlMessage<StopPositionMessage>),
    /// STP_QTDATA stop streaming message
    StopQtData(IgtlMessage<StopQtDataMessage>),
    /// STP_TDATA stop streaming message
    StopTData(IgtlMessage<StopTDataMessage>),
    /// STP_IMAGE stop streaming message
    StopImage(IgtlMessage<StopImageMessage>),
    /// STP_NDARRAY stop streaming message
    StopNdArray(IgtlMessage<StopNdArrayMessage>),

    /// Unknown message type (unrecognized or custom message)
    ///
    /// Contains the header and raw body bytes for manual processing.
    Unknown {
        /// Message header
        header: Header,
        /// Raw message body bytes
        body: Vec<u8>,
    },
}

impl AnyMessage {
    /// Get the message type name as a string
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use openigtlink_rust::protocol::AnyMessage;
    /// # use openigtlink_rust::protocol::types::TransformMessage;
    /// # use openigtlink_rust::protocol::message::IgtlMessage;
    /// # fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
    /// # let transform = TransformMessage::identity();
    /// # let msg = IgtlMessage::new(transform, "Device")?;
    /// # let any_msg = AnyMessage::Transform(msg);
    /// assert_eq!(any_msg.message_type(), "TRANSFORM");
    /// # Ok(())
    /// # }
    /// ```
    pub fn message_type(&self) -> &str {
        match self {
            AnyMessage::Transform(_) => "TRANSFORM",
            AnyMessage::Status(_) => "STATUS",
            AnyMessage::Capability(_) => "CAPABILITY",
            AnyMessage::Image(_) => "IMAGE",
            AnyMessage::Position(_) => "POSITION",
            AnyMessage::String(_) => "STRING",
            AnyMessage::QtData(_) => "QTDATA",
            AnyMessage::TData(_) => "TDATA",
            AnyMessage::Sensor(_) => "SENSOR",
            AnyMessage::Point(_) => "POINT",
            AnyMessage::Trajectory(_) => "TRAJECTORY",
            AnyMessage::NdArray(_) => "NDARRAY",
            AnyMessage::Bind(_) => "BIND",
            AnyMessage::ColorTable(_) => "COLORTABLE",
            AnyMessage::ImgMeta(_) => "IMGMETA",
            AnyMessage::LbMeta(_) => "LBMETA",
            AnyMessage::PolyData(_) => "POLYDATA",
            AnyMessage::Video(_) => "VIDEO",
            AnyMessage::VideoMeta(_) => "VIDEOMETA",
            AnyMessage::Command(_) => "COMMAND",
            AnyMessage::GetTransform(_) => "GET_TRANSFORM",
            AnyMessage::GetStatus(_) => "GET_STATUS",
            AnyMessage::GetCapability(_) => "GET_CAPABILITY",
            AnyMessage::GetImage(_) => "GET_IMAGE",
            AnyMessage::GetImgMeta(_) => "GET_IMGMETA",
            AnyMessage::GetLbMeta(_) => "GET_LBMETA",
            AnyMessage::GetPoint(_) => "GET_POINT",
            AnyMessage::GetTData(_) => "GET_TDATA",
            AnyMessage::RtsTransform(_) => "RTS_TRANSFORM",
            AnyMessage::RtsStatus(_) => "RTS_STATUS",
            AnyMessage::RtsCapability(_) => "RTS_CAPABILITY",
            AnyMessage::RtsImage(_) => "RTS_IMAGE",
            AnyMessage::RtsTData(_) => "RTS_TDATA",
            AnyMessage::StartTData(_) => "STT_TDATA",
            AnyMessage::StopTransform(_) => "STP_TRANSFORM",
            AnyMessage::StopPosition(_) => "STP_POSITION",
            AnyMessage::StopQtData(_) => "STP_QTDATA",
            AnyMessage::StopTData(_) => "STP_TDATA",
            AnyMessage::StopImage(_) => "STP_IMAGE",
            AnyMessage::StopNdArray(_) => "STP_NDARRAY",
            AnyMessage::Unknown { header, .. } => header.type_name.as_str().unwrap_or("UNKNOWN"),
        }
    }

    /// Get the device name from the message header
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use openigtlink_rust::protocol::AnyMessage;
    /// # use openigtlink_rust::protocol::types::TransformMessage;
    /// # use openigtlink_rust::protocol::message::IgtlMessage;
    /// # fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
    /// # let transform = TransformMessage::identity();
    /// # let msg = IgtlMessage::new(transform, "MyDevice")?;
    /// # let any_msg = AnyMessage::Transform(msg);
    /// assert_eq!(any_msg.device_name()?, "MyDevice");
    /// # Ok(())
    /// # }
    /// ```
    pub fn device_name(&self) -> Result<&str> {
        let header = match self {
            AnyMessage::Transform(msg) => &msg.header,
            AnyMessage::Status(msg) => &msg.header,
            AnyMessage::Capability(msg) => &msg.header,
            AnyMessage::Image(msg) => &msg.header,
            AnyMessage::Position(msg) => &msg.header,
            AnyMessage::String(msg) => &msg.header,
            AnyMessage::QtData(msg) => &msg.header,
            AnyMessage::TData(msg) => &msg.header,
            AnyMessage::Sensor(msg) => &msg.header,
            AnyMessage::Point(msg) => &msg.header,
            AnyMessage::Trajectory(msg) => &msg.header,
            AnyMessage::NdArray(msg) => &msg.header,
            AnyMessage::Bind(msg) => &msg.header,
            AnyMessage::ColorTable(msg) => &msg.header,
            AnyMessage::ImgMeta(msg) => &msg.header,
            AnyMessage::LbMeta(msg) => &msg.header,
            AnyMessage::PolyData(msg) => &msg.header,
            AnyMessage::Video(msg) => &msg.header,
            AnyMessage::VideoMeta(msg) => &msg.header,
            AnyMessage::Command(msg) => &msg.header,
            AnyMessage::GetTransform(msg) => &msg.header,
            AnyMessage::GetStatus(msg) => &msg.header,
            AnyMessage::GetCapability(msg) => &msg.header,
            AnyMessage::GetImage(msg) => &msg.header,
            AnyMessage::GetImgMeta(msg) => &msg.header,
            AnyMessage::GetLbMeta(msg) => &msg.header,
            AnyMessage::GetPoint(msg) => &msg.header,
            AnyMessage::GetTData(msg) => &msg.header,
            AnyMessage::RtsTransform(msg) => &msg.header,
            AnyMessage::RtsStatus(msg) => &msg.header,
            AnyMessage::RtsCapability(msg) => &msg.header,
            AnyMessage::RtsImage(msg) => &msg.header,
            AnyMessage::RtsTData(msg) => &msg.header,
            AnyMessage::StartTData(msg) => &msg.header,
            AnyMessage::StopTransform(msg) => &msg.header,
            AnyMessage::StopPosition(msg) => &msg.header,
            AnyMessage::StopQtData(msg) => &msg.header,
            AnyMessage::StopTData(msg) => &msg.header,
            AnyMessage::StopImage(msg) => &msg.header,
            AnyMessage::StopNdArray(msg) => &msg.header,
            AnyMessage::Unknown { header, .. } => header,
        };
        header.device_name.as_str()
    }

    /// Get reference to the message header
    pub fn header(&self) -> &Header {
        match self {
            AnyMessage::Transform(msg) => &msg.header,
            AnyMessage::Status(msg) => &msg.header,
            AnyMessage::Capability(msg) => &msg.header,
            AnyMessage::Image(msg) => &msg.header,
            AnyMessage::Position(msg) => &msg.header,
            AnyMessage::String(msg) => &msg.header,
            AnyMessage::QtData(msg) => &msg.header,
            AnyMessage::TData(msg) => &msg.header,
            AnyMessage::Sensor(msg) => &msg.header,
            AnyMessage::Point(msg) => &msg.header,
            AnyMessage::Trajectory(msg) => &msg.header,
            AnyMessage::NdArray(msg) => &msg.header,
            AnyMessage::Bind(msg) => &msg.header,
            AnyMessage::ColorTable(msg) => &msg.header,
            AnyMessage::ImgMeta(msg) => &msg.header,
            AnyMessage::LbMeta(msg) => &msg.header,
            AnyMessage::PolyData(msg) => &msg.header,
            AnyMessage::Video(msg) => &msg.header,
            AnyMessage::VideoMeta(msg) => &msg.header,
            AnyMessage::Command(msg) => &msg.header,
            AnyMessage::GetTransform(msg) => &msg.header,
            AnyMessage::GetStatus(msg) => &msg.header,
            AnyMessage::GetCapability(msg) => &msg.header,
            AnyMessage::GetImage(msg) => &msg.header,
            AnyMessage::GetImgMeta(msg) => &msg.header,
            AnyMessage::GetLbMeta(msg) => &msg.header,
            AnyMessage::GetPoint(msg) => &msg.header,
            AnyMessage::GetTData(msg) => &msg.header,
            AnyMessage::RtsTransform(msg) => &msg.header,
            AnyMessage::RtsStatus(msg) => &msg.header,
            AnyMessage::RtsCapability(msg) => &msg.header,
            AnyMessage::RtsImage(msg) => &msg.header,
            AnyMessage::RtsTData(msg) => &msg.header,
            AnyMessage::StartTData(msg) => &msg.header,
            AnyMessage::StopTransform(msg) => &msg.header,
            AnyMessage::StopPosition(msg) => &msg.header,
            AnyMessage::StopQtData(msg) => &msg.header,
            AnyMessage::StopTData(msg) => &msg.header,
            AnyMessage::StopImage(msg) => &msg.header,
            AnyMessage::StopNdArray(msg) => &msg.header,
            AnyMessage::Unknown { header, .. } => header,
        }
    }

    /// Try to extract as a Transform message
    pub fn as_transform(&self) -> Option<&IgtlMessage<TransformMessage>> {
        match self {
            AnyMessage::Transform(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to extract as a Status message
    pub fn as_status(&self) -> Option<&IgtlMessage<StatusMessage>> {
        match self {
            AnyMessage::Status(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to extract as an Image message
    pub fn as_image(&self) -> Option<&IgtlMessage<ImageMessage>> {
        match self {
            AnyMessage::Image(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to extract as a Position message
    pub fn as_position(&self) -> Option<&IgtlMessage<PositionMessage>> {
        match self {
            AnyMessage::Position(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to extract as a String message
    pub fn as_string(&self) -> Option<&IgtlMessage<StringMessage>> {
        match self {
            AnyMessage::String(msg) => Some(msg),
            _ => None,
        }
    }

    /// Try to extract as a Capability message
    pub fn as_capability(&self) -> Option<&IgtlMessage<CapabilityMessage>> {
        match self {
            AnyMessage::Capability(msg) => Some(msg),
            _ => None,
        }
    }

    /// Check if this is an unknown message type
    pub fn is_unknown(&self) -> bool {
        matches!(self, AnyMessage::Unknown { .. })
    }

    /// Decode a message from raw bytes with optional CRC verification
    ///
    /// This is a lower-level method that attempts to decode the message
    /// based on its type_name field in the header.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw message bytes (header + body)
    /// * `verify_crc` - Whether to verify CRC checksum
    ///
    /// # Errors
    ///
    /// - [`IgtlError::InvalidHeader`](crate::error::IgtlError::InvalidHeader) - Malformed header
    /// - [`IgtlError::CrcMismatch`](crate::error::IgtlError::CrcMismatch) - CRC verification failed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use openigtlink_rust::protocol::AnyMessage;
    /// # fn example() -> Result<(), openigtlink_rust::error::IgtlError> {
    /// # let data: Vec<u8> = vec![];
    /// let msg = AnyMessage::decode_with_options(&data, true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode_with_options(data: &[u8], verify_crc: bool) -> Result<Self> {
        use crate::error::IgtlError;

        // Decode header first to determine message type
        let header = Header::decode(&data[..Header::SIZE])?;
        let type_name = header.type_name.as_str()?;

        // Try to decode as specific message types based on type_name
        match type_name {
            "TRANSFORM" => {
                if let Ok(msg) = IgtlMessage::<TransformMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::Transform(msg));
                }
            }
            "STATUS" => {
                if let Ok(msg) = IgtlMessage::<StatusMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::Status(msg));
                }
            }
            "CAPABILITY" => {
                if let Ok(msg) = IgtlMessage::<CapabilityMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::Capability(msg));
                }
            }
            "IMAGE" => {
                if let Ok(msg) = IgtlMessage::<ImageMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::Image(msg));
                }
            }
            "POSITION" => {
                if let Ok(msg) = IgtlMessage::<PositionMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::Position(msg));
                }
            }
            "STRING" => {
                if let Ok(msg) = IgtlMessage::<StringMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::String(msg));
                }
            }
            "QTDATA" => {
                if let Ok(msg) = IgtlMessage::<QtDataMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::QtData(msg));
                }
            }
            "TDATA" => {
                if let Ok(msg) = IgtlMessage::<TDataMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::TData(msg));
                }
            }
            "SENSOR" => {
                if let Ok(msg) = IgtlMessage::<SensorMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::Sensor(msg));
                }
            }
            "POINT" => {
                if let Ok(msg) = IgtlMessage::<PointMessage>::decode_with_options(data, verify_crc) {
                    return Ok(AnyMessage::Point(msg));
                }
            }
            _ => {
                // Unknown message type - store header and body
                let body = data[Header::SIZE..].to_vec();
                return Ok(AnyMessage::Unknown { header, body });
            }
        }

        // If we get here, the message type matched but decode failed
        Err(IgtlError::UnknownMessageType(type_name.to_string()))
    }
}
