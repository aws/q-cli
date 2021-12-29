//! Local Protocal Buffers

#![allow(clippy::all)]

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use prost::Message;

include!(concat!(env!("OUT_DIR"), "/local.rs"));

impl LocalMessage {
    pub fn to_fig_pbuf(&self) -> Result<Bytes> {
        let mut fig_pbuf = BytesMut::new();

        let mut encoded_message = BytesMut::new();
        self.encode(&mut encoded_message)?;

        fig_pbuf.extend(b"\x1b@fig-pbuf");
        fig_pbuf.extend(encoded_message.len().to_be_bytes());
        fig_pbuf.extend(encoded_message);

        Ok(fig_pbuf.freeze())
    }
}
