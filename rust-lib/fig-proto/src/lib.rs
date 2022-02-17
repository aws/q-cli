//! Protocal buffer definitions

pub mod daemon;
pub mod figterm;
pub mod hooks;
pub mod local;
pub use prost;

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use prost::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FigMessageType {
    Protobuf,
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
    inner: Bytes,
    _message_type: FigMessageType,
}

impl std::ops::Deref for FigMessage {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A trait for types that can be converted to a FigProtobuf
pub trait FigProtobufEncodable {
    /// Encodes a protobuf message into a fig message
    fn encode_fig_protobuf(&self) -> Result<FigMessage>;
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
            _message_type: FigMessageType::Protobuf,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_fig_pbuf() {
        let hook = hooks::new_edit_buffer_hook(None, "test".into(), 0, 0);
        let message = hooks::hook_to_message(hook);

        assert!(message
            .encode_fig_protobuf()
            .unwrap()
            .starts_with(b"\x1b@fig-pbuf"));

        assert_eq!(
            message.encode_fig_protobuf().unwrap()._message_type,
            FigMessageType::Protobuf
        );
    }
}
