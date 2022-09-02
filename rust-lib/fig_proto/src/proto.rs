#![allow(clippy::all)]

/// Fig Common Protocol Buffers
///
/// These are shared definition used in the other
/// protocol buffer definition.
pub(crate) mod fig_common {
    include!(concat!(env!("OUT_DIR"), "/fig_common.rs"));
}

/// Fig.js Protocol Buffers
pub(crate) mod fig {
    pub use super::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/fig.rs"));
}

/// Local Protocol Buffers
pub(crate) mod local {
    pub use super::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/local.rs"));
}

/// Daemon Protocol Buffers
pub(crate) mod daemon {
    pub use super::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/daemon.rs"));
}

/// Figterm Protocol Buffers
pub(crate) mod figterm {
    pub use crate::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/figterm.rs"));
}

/// Secure Socket Protocol Buffers
pub(crate) mod secure {
    pub use crate::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/secure.rs"));
}
