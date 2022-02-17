//! Local Protocal Buffers

mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/local.rs"));
}

pub use proto::*;
