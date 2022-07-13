mod alias;
mod daemon_dispatch;
mod error;
mod identify;
mod install_method;
pub mod sentry;
mod track;
mod util;

pub use alias::emit_alias;
pub use daemon_dispatch::dispatch_emit_track;
pub use error::Error;
pub use identify::emit_identify;
pub use track::{
    emit_track,
    TrackEvent,
    TrackSource,
};

pub use crate::sentry::init_sentry;

const API_DOMAIN: &str = "https://api.fig.io/telemetry/";
const ALIAS_SUBDOMAIN: &str = "alias";
const IDENTIFY_SUBDOMAIN: &str = "identify";
const TRACK_SUBDOMAIN: &str = "track";
