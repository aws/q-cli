//! Local Protocal Buffers

use prost::Message;

include!(concat!(env!("OUT_DIR"), "/local.rs"));

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
