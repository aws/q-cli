//! Protocal buffer definitions

pub mod figterm;
pub mod hooks;
pub mod local;

use bytes::{Bytes, BytesMut};
pub use local::*;

use prost::Message;

impl LocalMessage {
    pub fn to_fig_pbuf(&self) -> Bytes {
        let mut fig_pbuf = BytesMut::new();

        let mut encoded_message = BytesMut::new();
        self.encode(&mut encoded_message);

        fig_pbuf.extend(b"\x1b@fig-pbuf");
        fig_pbuf.extend(encoded_message.len().to_be_bytes());
        fig_pbuf.extend(encoded_message);

        fig_pbuf.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_fig_pbuf() {
        let hook = hooks::new_edit_buffer_hook(None, "test".into(), 0, 0);
        let message = hooks::hook_to_message(hook);
        assert!(message.to_fig_pbuf().starts_with(b"\x1b@fig-pbuf"))
    }
}
