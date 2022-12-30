mod daemon_dispatch;
mod error;
pub mod feature_flags;
mod identify;
mod install_method;
mod page;
pub mod sentry;
mod track;
mod util;

pub use daemon_dispatch::dispatch_emit_track;
pub use error::Error;
pub use identify::emit_identify;
pub use install_method::{
    get_install_method,
    InstallMethod,
};
pub use page::emit_page;
pub use track::{
    emit_track,
    emit_tracks,
    TrackEvent,
    TrackEventType,
    TrackSource,
};

pub use crate::sentry::init_sentry;

const IDENTIFY_SUBDOMAIN: &str = "/telemetry/identify";
const TRACK_SUBDOMAIN: &str = "/telemetry/track";
const PAGE_SUBDOMAIN: &str = "/telemetry/page";
