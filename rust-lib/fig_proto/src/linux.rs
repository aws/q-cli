//! Linux Protocol Buffers

mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/linux.rs"));
}

pub use proto::*;
