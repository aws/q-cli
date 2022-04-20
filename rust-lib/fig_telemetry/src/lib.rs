mod alias;
mod error;
mod identify;
mod install_method;
mod track;

pub use alias::emit_alias;
pub use error::Error;
pub use identify::emit_identify;
pub use track::{emit_track, TrackEvent, TrackSource};

const API_DOMAIN: &str = "https://tel.withfig.com/";
const ALIAS_SUBDOMAIN: &str = "alias";
const IDENTIFY_SUBDOMAIN: &str = "identify";
const TRACK_SUBDOMAIN: &str = "track";
