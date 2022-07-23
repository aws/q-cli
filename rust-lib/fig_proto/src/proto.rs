#![allow(clippy::all)]

/// Fig Common Protocal Buffers
///
/// These are shared definition used in the other
/// protocal buffer definition.
pub(crate) mod fig_common {
    include!(concat!(env!("OUT_DIR"), "/fig_common.rs"));
}

/// Fig.js Protocal Buffers
pub(crate) mod fig {
    pub use super::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/fig.rs"));
}

/// Local Protocal Buffers
pub(crate) mod local {
    pub use super::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/local.rs"));
}

/// Daemon Protocal Buffers
pub(crate) mod daemon {
    pub use super::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/daemon.rs"));
}

/// Figterm Protocal Buffers
pub(crate) mod figterm {
    pub use crate::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/figterm.rs"));
}
