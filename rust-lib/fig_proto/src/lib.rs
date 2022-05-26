//! Protocal buffer definitions

pub mod daemon;
pub mod fig;
pub mod figterm;
pub mod hooks;
pub mod local;
pub mod util;

use std::fmt::Debug;
use std::io::{
    Cursor,
    Read,
};
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
use prost_reflect::{
    DescriptorPool,
    DynamicMessage,
    ReflectMessage,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Deserializer;
use thiserror::Error;

static DESCRIPTOR_POOL: Lazy<DescriptorPool> = Lazy::new(|| {
    DescriptorPool::decode(include_bytes!(concat!(env!("OUT_DIR"), "/file_descriptor_set.bin")).as_ref()).unwrap()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FigMessageType {
    Protobuf,
    Json,
}

/// A fig message
///
/// The format of a fig message is:
///
///   - The header `\x1b@`
///   - The type of the message, in this case `fig-pbuf`, this part is always 8 bytes
///   - The length of the remainder of the message encoded as a big endian u64
///   - The message, in this case a protobuf message
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
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum FigMessageDecodeError {
    #[error("prost decode error: {0}")]
    ProstDecode(#[from] DecodeError),
    #[error("json decode error: {0}")]
    JsonDecode(#[from] serde_json::Error),
    #[error("name is a valid protobuf: {0}")]
    NameNotValid(String),
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
        src.read_exact(&mut header)?;

        if header[0] != b'\x1b' || header[1] != b'@' {
            return Err(FigMessageParseError::InvalidHeader);
        }

        let mut message_type_buf = [0; 8];
        src.read_exact(&mut message_type_buf)?;

        let message_type = match &message_type_buf {
            b"fig-pbuf" => FigMessageType::Protobuf,
            b"fig-json" => FigMessageType::Json,
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
        src.read_exact(&mut inner)?;

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
                &mut Deserializer::from_slice(self.inner.as_ref()),
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
    fn json_decode() {
        let message = test_message();
        let json = serde_json::to_vec(&message.transcode_to_dynamic()).unwrap();

        let msg = FigMessage {
            inner: Bytes::from(json),
            message_type: FigMessageType::Json,
        };
        let decoded_message: local::LocalMessage = msg.decode().unwrap();

        assert_eq!(message, decoded_message);
    }
}
