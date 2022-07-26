mod alias;
mod daemon_dispatch;
mod error;
mod identify;
mod install_method;
mod page;
pub mod sentry;
mod track;
mod util;

pub use alias::emit_alias;
pub use daemon_dispatch::dispatch_emit_track;
pub use error::Error;
pub use identify::emit_identify;
pub use page::emit_page;
pub use track::{
    emit_track,
    emit_tracks,
    TrackEvent,
    TrackEventType,
    TrackSource,
};

pub use crate::sentry::init_sentry;

const API_DOMAIN: &str = "https://api.fig.io/telemetry/";
const ALIAS_SUBDOMAIN: &str = "alias";
const IDENTIFY_SUBDOMAIN: &str = "identify";
const TRACK_SUBDOMAIN: &str = "track";
const PAGE_SUBDOMAIN: &str = "page";
