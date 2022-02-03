//! Local Protocal Buffers

#![allow(clippy::all)]

use anyhow::Result;
use bytes::BytesMut;
use prost::Message;

use super::{FigProtobuf, FigProtobufEncodable};

include!(concat!(env!("OUT_DIR"), "/local.rs"));

impl FigProtobufEncodable for LocalMessage {
    fn encode_fig_protobuf(&self) -> Result<FigProtobuf> {
        let mut fig_pbuf = BytesMut::new();

        let mut encoded_message = BytesMut::new();
        self.encode(&mut encoded_message)?;

        fig_pbuf.extend(b"\x1b@fig-pbuf");
        fig_pbuf.extend(encoded_message.len().to_be_bytes());
        fig_pbuf.extend(encoded_message);

        Ok(FigProtobuf {
            inner: fig_pbuf.freeze(),
        })
    }
}

