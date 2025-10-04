//! Query and streaming control messages
//!
//! This module implements OpenIGTLink query and streaming control messages
//! for C++ OpenIGTLink server compatibility (3D Slicer, PLUS Toolkit, etc.)
//!
//! # Message Types
//!
//! - **GET_***: Request single data (e.g., GET_CAPABILITY, GET_STATUS)
//! - **STT_***: Start streaming (e.g., STT_TDATA)
//! - **STP_***: Stop streaming (e.g., STP_TDATA)
//! - **RTS_***: Ready-to-send response (e.g., RTS_TDATA)


pub mod get;
pub mod rts;
pub mod streaming;

// Re-export query message types
pub use get::*;
pub use rts::{
    RtsCapabilityMessage, RtsImageMessage, RtsStatusMessage, RtsTDataMessage,
    RtsTransformMessage,
};
pub use streaming::{
    StartTDataMessage, StopImageMessage, StopNdArrayMessage, StopPositionMessage,
    StopQtDataMessage, StopTDataMessage, StopTransformMessage,
};

/// Macro to define empty-body query messages (GET_*, STP_*)
///
/// # Usage
///
/// ```ignore
/// impl_empty_query!(GetTransformMessage, "GET_TRANSFOR");
/// impl_empty_query!(StopTDataMessage, "STP_TDATA");
/// ```
macro_rules! impl_empty_query {
    ($name:ident, $type_str:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name;

        impl $crate::protocol::message::Message for $name {
            fn message_type() -> &'static str {
                $type_str
            }

            fn encode_content(&self) -> $crate::error::Result<Vec<u8>> {
                Ok(vec![])
            }

            fn decode_content(_data: &[u8]) -> $crate::error::Result<Self> {
                Ok(Self)
            }
        }
    };
}

pub(crate) use impl_empty_query;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::message::Message;

    // Test macro expansion
    impl_empty_query!(TestQueryMessage, "TEST_QUERY");

    #[test]
    fn test_empty_query_macro() {
        assert_eq!(TestQueryMessage::message_type(), "TEST_QUERY");

        let msg = TestQueryMessage;
        let encoded = msg.encode_content().unwrap();
        assert_eq!(encoded.len(), 0);

        let decoded = TestQueryMessage::decode_content(&[]).unwrap();
        assert_eq!(msg, decoded);
    }
}
