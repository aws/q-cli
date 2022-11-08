//! Protocol buffer definitions

pub mod daemon;
pub mod fig;
pub mod fig_common;
pub mod figterm;
pub mod hooks;
pub mod local;
pub(crate) mod proto;
pub mod secure_hooks;
pub mod stress;
pub mod util;
use std::fmt::Debug;
use std::mem::size_of;

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
use prost_reflect::DescriptorPool;
pub use prost_reflect::{
    DynamicMessage,
    ReflectMessage,
};
use serde::Serialize;
use thiserror::Error;

pub mod secure {
    pub use crate::proto::secure::*;
}

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

impl FigMessageType {
    pub const fn header(&self) -> &'static [u8] {
        match self {
            FigMessageType::Protobuf => b"fig-pbuf",
            FigMessageType::Json => b"fig-json",
            FigMessageType::MessagePack => b"fig-mpak",
        }
    }
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

#[derive(Debug)]
pub enum FigMessageComponent {
    Header,
    BodySize,
    Body,
}

#[derive(Debug, Error)]
pub enum FigMessageParseError {
    #[error("incomplete message, missing {0:?}")]
    Incomplete(FigMessageComponent),
    #[error("invalid message header {0} (raw type {1})")]
    InvalidHeader(String, String),
    #[error("invalid message type")]
    InvalidMessageType([u8; 8]),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum FigMessageDecodeError {
    #[error("name is not a valid protobuf: {0}")]
    NameNotValid(String),
    #[error(transparent)]
    ProstDecode(#[from] DecodeError),
    #[error(transparent)]
    JsonDecode(#[from] serde_json::Error),
    #[error(transparent)]
    RmpDecode(#[from] rmp_serde::decode::Error),
}

#[derive(Debug, Error)]
pub enum FigMessageEncodeError {
    #[error(transparent)]
    IntError(#[from] std::num::TryFromIntError),
    #[error(transparent)]
    JsonEncode(#[from] serde_json::Error),
    #[error(transparent)]
    RmpEncode(#[from] rmp_serde::encode::Error),
}

impl FigMessage {
    pub fn json(json: impl Serialize) -> Result<Bytes, FigMessageEncodeError> {
        FigMessage::encode(FigMessageType::Json, &serde_json::to_vec(&json)?)
    }

    pub fn message_pack(message_pack: impl Serialize) -> Result<Bytes, FigMessageEncodeError> {
        FigMessage::encode(FigMessageType::MessagePack, &rmp_serde::to_vec(&message_pack)?)
    }

    pub fn encode(message_type: FigMessageType, body: &[u8]) -> Result<Bytes, FigMessageEncodeError> {
        let message_len: u64 = body.len().try_into()?;
        let message_len_be = message_len.to_be_bytes();

        let mut inner =
            BytesMut::with_capacity(b"\x1b@".len() + message_type.header().len() + message_len_be.len() + body.len());

        inner.extend_from_slice(b"\x1b@");
        inner.extend_from_slice(message_type.header());
        inner.extend_from_slice(&message_len_be);
        inner.extend_from_slice(body);

        Ok(inner.freeze())
    }

    pub fn parse(src: &mut impl bytes::Buf) -> Result<(usize, FigMessage), FigMessageParseError> {
        if src.remaining() < 10 {
            return Err(FigMessageParseError::Incomplete(FigMessageComponent::Header));
        }

        let mut header = [0; 2];
        src.copy_to_slice(&mut header);
        if header[0] != b'\x1b' || header[1] != b'@' {
            let mut message_type_buf = [0; 8];
            src.copy_to_slice(&mut message_type_buf);
            return Err(FigMessageParseError::InvalidHeader(
                hex::encode(header),
                hex::encode(message_type_buf),
            ));
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
            return Err(FigMessageParseError::Incomplete(FigMessageComponent::BodySize));
        }

        let len = src.get_u64();

        if src.remaining() < len as usize {
            return Err(FigMessageParseError::Incomplete(FigMessageComponent::Body));
        }

        let mut inner = vec![0; len as usize];
        src.copy_to_slice(&mut inner);

        let message_len = 10 + size_of::<u64>() + len as usize;

        Ok((message_len, FigMessage {
            inner: Bytes::from(inner),
            message_type,
        }))
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
    fn encode_fig_protobuf(&self) -> Result<Bytes, FigMessageEncodeError>;
}

impl<T: Message> FigProtobufEncodable for T {
    fn encode_fig_protobuf(&self) -> Result<Bytes, FigMessageEncodeError> {
        FigMessage::encode(FigMessageType::Protobuf, &self.encode_to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_message() -> local::LocalMessage {
        let ctx = local::ShellContext {
            pid: Some(123),
            ttys: Some("/dev/pty123".into()),
            process_name: Some("/bin/bash".into()),
            current_working_directory: Some("/home/user".into()),
            session_id: None,
            terminal: None,
            hostname: None,
            shell_path: Some("/bin/bash".into()),
            wsl_distro: None,
            environment_variables: vec![],
            figterm_version: None,
            preexec: Some(false),
            osc_lock: Some(true),
        };
        let hook = hooks::new_edit_buffer_hook(Some(ctx), "test", 2, 3, None);
        hooks::hook_to_message(hook)
    }

    #[test]
    fn test_to_fig_pbuf() {
        let message = test_message();
        assert!(message.encode_fig_protobuf().unwrap().starts_with(b"\x1b@fig-pbuf"));
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
    "caretPosition": {
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

        let caret_position = match hook.hook.unwrap() {
            local::hook::Hook::CaretPosition(caret_position) => caret_position,
            _ => panic!(),
        };

        assert_eq!(caret_position.x, 123.0);
        assert_eq!(caret_position.y, 456.0);
        assert_eq!(caret_position.width, 34.0);
        assert_eq!(caret_position.height, 61.0);
    }

    #[test]
    fn rmp_round_trip() {
        let message = test_message();
        let mpack = rmp_serde::to_vec(&message.transcode_to_dynamic()).unwrap();

        let msg = FigMessage {
            inner: Bytes::from(mpack),
            message_type: FigMessageType::MessagePack,
        };
        let decoded_message: local::LocalMessage = msg.decode().unwrap();

        assert_eq!(message, decoded_message);
    }
}
