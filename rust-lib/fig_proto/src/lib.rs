//! Protocal buffer definitions

pub mod daemon;
pub mod fig;
pub mod fig_common;
pub mod figterm;
pub mod hooks;
pub mod local;
pub(crate) mod proto;
pub mod util;

use std::fmt::Debug;
use std::io::Cursor;
use std::mem::size_of;

use anyhow::Result;
use bytes::{
    Buf,
    Bytes,
    BytesMut,
};
use once_cell::sync::Lazy;
pub use prost;
use prost::{
    DecodeError,
    Message,
};
pub use prost_reflect::ReflectMessage;
use prost_reflect::{
    DescriptorPool,
    DynamicMessage,
};
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

// This is not used explicitly, but it must be here for the derive
// impls on the protos for dynamic message
static DESCRIPTOR_POOL: Lazy<DescriptorPool> = Lazy::new(|| {
    DescriptorPool::decode(include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin")).as_ref()).unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigMessageType {
    Protobuf,
    Json,
    MessagePack,
}

/// A fig message
///
/// The format of a fig message is:
///
///   - The header `\x1b@`
///   - The type of the message (must be 8 bytes)
///     - `fig-pbuf` - Protocol Buffer
///     - `fig-json` - Json
///     - `fig-mpak` - MessagePack
///   - The length of the remainder of the message encoded as a big endian u64
///   - The message, encoded as protobuf, json-protobuf, or messagepack-protobuf
#[derive(Debug, Clone)]
pub struct FigMessage {
    pub inner: Bytes,
    pub message_type: FigMessageType,
}

#[derive(Debug, Error)]
pub enum FigMessageParseError {
    #[error("incomlete message")]
    Incomplete,
    #[error("invalid message header")]
    InvalidHeader,
    #[error("invalid message type")]
    InvalidMessageType([u8; 8]),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum FigMessageDecodeError {
    #[error("name is a valid protobuf: {0}")]
    NameNotValid(String),
    #[error(transparent)]
    ProstDecode(#[from] DecodeError),
    #[error(transparent)]
    JsonDecode(#[from] serde_json::Error),
    #[error(transparent)]
    RmpDecode(#[from] rmp_serde::decode::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct FigJsonMessage {
    name: String,
    data: serde_json::Value,
}

impl FigMessage {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<FigMessage, FigMessageParseError> {
        if src.remaining() < 10 {
            return Err(FigMessageParseError::Incomplete);
        }

        let mut header = [0; 2];
        src.copy_to_slice(&mut header);
        if header[0] != b'\x1b' || header[1] != b'@' {
            return Err(FigMessageParseError::InvalidHeader);
        }

        let mut message_type_buf = [0; 8];
        src.copy_to_slice(&mut message_type_buf);
        let message_type = match &message_type_buf {
            b"fig-pbuf" => FigMessageType::Protobuf,
            b"fig-json" => FigMessageType::Json,
            b"fig-mpak" => FigMessageType::MessagePack,
            _ => return Err(FigMessageParseError::InvalidMessageType(message_type_buf)),
        };

        if src.remaining() < size_of::<u64>() {
            return Err(FigMessageParseError::Incomplete);
        }

        let len = src.get_u64();

        if src.remaining() < len as usize {
            return Err(FigMessageParseError::Incomplete);
        }

        let mut inner = vec![0; len as usize];
        src.copy_to_slice(&mut inner);

        Ok(FigMessage {
            inner: Bytes::from(inner),
            message_type,
        })
    }

    pub fn decode<T>(self) -> Result<T, FigMessageDecodeError>
    where
        T: Message + ReflectMessage + Default,
    {
        match self.message_type {
            FigMessageType::Protobuf => Ok(T::decode(self.inner)?),
            FigMessageType::Json => Ok(DynamicMessage::deserialize(
                T::default().descriptor(),
                &mut serde_json::Deserializer::from_slice(self.inner.as_ref()),
            )?
            .transcode_to()?),
            FigMessageType::MessagePack => Ok(DynamicMessage::deserialize(
                T::default().descriptor(),
                &mut rmp_serde::Deserializer::from_read_ref(self.inner.as_ref()),
            )?
            .transcode_to()?),
        }
    }
}

impl std::ops::Deref for FigMessage {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A trait for types that can be converted to a FigProtobuf
pub trait FigProtobufEncodable: Debug + Send + Sync {
    /// Encodes a protobuf message into a fig message
    fn encode_fig_protobuf(&self) -> Result<FigMessage>;
}

impl FigProtobufEncodable for FigMessage {
    fn encode_fig_protobuf(&self) -> Result<FigMessage> {
        Ok(self.clone())
    }
}

impl<T: Message> FigProtobufEncodable for T {
    fn encode_fig_protobuf(&self) -> Result<FigMessage> {
        let mut fig_pbuf = BytesMut::new();

        let mut encoded_message = BytesMut::new();
        self.encode(&mut encoded_message)?;

        let message_len: u64 = encoded_message.len().try_into()?;

        fig_pbuf.extend(b"\x1b@fig-pbuf");
        fig_pbuf.extend(message_len.to_be_bytes());
        fig_pbuf.extend(encoded_message);

        Ok(FigMessage {
            inner: fig_pbuf.freeze(),
            message_type: FigMessageType::Protobuf,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_message() -> local::LocalMessage {
        let ctx = hooks::new_context(
            Some(123),
            Some("/dev/pty123".into()),
            Some("/bin/bash".into()),
            Some("/home/user".into()),
            None,
            None,
            None,
            None,
        );
        let hook = hooks::new_edit_buffer_hook(Some(ctx), "test", 2, 3);
        hooks::hook_to_message(hook)
    }

    #[test]
    fn test_to_fig_pbuf() {
        let message = test_message();

        assert!(message.encode_fig_protobuf().unwrap().starts_with(b"\x1b@fig-pbuf"));

        assert_eq!(
            message.encode_fig_protobuf().unwrap().message_type,
            FigMessageType::Protobuf
        );
    }

    #[test]
    fn json_round_trip() {
        let message = test_message();
        let json = serde_json::to_vec(&message.transcode_to_dynamic()).unwrap();

        let msg = FigMessage {
            inner: Bytes::from(json),
            message_type: FigMessageType::Json,
        };
        let decoded_message: local::LocalMessage = msg.decode().unwrap();

        assert_eq!(message, decoded_message);
    }

    #[test]
    fn json_decode() {
        let msg = FigMessage {
            inner: Bytes::from(
                r#"{
  "hook": {
    "cursorPosition": {
      "x": 123,
      "y": 456,
      "width": 34,
      "height": 61
    }
  }
}"#,
            ),
            message_type: FigMessageType::Json,
        };

        let decoded_message: local::LocalMessage = msg.decode().unwrap();

        let hook = match decoded_message.r#type.unwrap() {
            local::local_message::Type::Hook(hook) => hook,
            _ => panic!(),
        };

        let cursor_position = match hook.hook.unwrap() {
            local::hook::Hook::CursorPosition(cursor_position) => cursor_position,
            _ => panic!(),
        };

        assert_eq!(cursor_position.x, 123);
        assert_eq!(cursor_position.y, 456);
        assert_eq!(cursor_position.width, 34);
        assert_eq!(cursor_position.height, 61);
    }

    #[test]
    fn rmp_round_trip() {
        let message = test_message();
        let json = rmp_serde::to_vec(&message.transcode_to_dynamic()).unwrap();

        let msg = FigMessage {
            inner: Bytes::from(json),
            message_type: FigMessageType::MessagePack,
        };
        let decoded_message: local::LocalMessage = msg.decode().unwrap();

        assert_eq!(message, decoded_message);
    }
}
