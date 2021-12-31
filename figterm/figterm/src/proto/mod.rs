//! Protocal buffer definitions

use bytes::Bytes;
use anyhow::Result;

pub mod figterm;
pub mod hooks;
pub mod local;

#[derive(Debug, Clone)]
pub struct FigProtobuf {
    inner: Bytes,
}

impl std::ops::Deref for FigProtobuf {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub trait FigProtobufEncodable {
    fn encode_fig_protobuf(&self) -> Result<FigProtobuf>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_fig_pbuf() {
        let hook = hooks::new_edit_buffer_hook(None, "test".into(), 0, 0);
        let message = hooks::hook_to_message(hook);
        assert!(message.encode_fig_protobuf().unwrap().starts_with(b"\x1b@fig-pbuf"))
    }
}
