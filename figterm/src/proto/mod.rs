//! Protocal buffer definitions

pub mod local;
pub mod hooks;

pub use local::*;

use prost::Message;

impl LocalMessage {
    pub fn to_fig_pbuf(&self) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::with_capacity(1024);

        let encoded_message = self.encode_to_vec();

        packet.extend(b"\x1b@fig-pbuf");
        packet.extend(encoded_message.len().to_be_bytes());
        packet.extend(encoded_message);

        packet
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