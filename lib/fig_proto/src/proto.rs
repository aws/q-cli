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

/// Figterm Protocol Buffers
pub(crate) mod figterm {
    pub use crate::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/figterm.rs"));
}

/// remote Socket Protocol Buffers
pub(crate) mod remote {
    pub use crate::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/remote.rs"));
}

/// Stress Testing Protocol Buffers
pub(crate) mod stress {
    pub use crate::fig_common::*;
    include!(concat!(env!("OUT_DIR"), "/stress.rs"));
}
