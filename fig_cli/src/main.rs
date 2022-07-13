#[macro_use]
extern crate cfg_if;

pub mod cli;
pub mod daemon;
pub mod util;

use std::process::exit;
use std::str::FromStr;

use clap::StructOpt;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() {
    let env_level = std::env::var("FIG_LOG_LEVEL")
        .ok()
        .and_then(|level| LevelFilter::from_str(&level).ok())
        .unwrap_or(LevelFilter::INFO);

    // Whitelist commands do not have sentry or telemetry, telemetry should only run on
    // user facing commands as performance is less important
    let (_guard, track_join) = match std::env::args().nth(1).as_deref() {
        Some("init" | "_" | "internal" | "tips" | "completion" | "hook") => (None, None),
        _ => (
            Some(fig_telemetry::init_sentry(
                "https://0631fceb9ae540bb874af81820507ebf@o436453.ingest.sentry.io/6187837",
            )),
            Some(fig_telemetry::dispatch_emit_track(
                fig_telemetry::TrackEvent::RanCommand,
                fig_telemetry::TrackSource::Cli,
                dbg!([
                    ("arguments", std::env::args().collect::<Vec<_>>().join(" ")),
                    (
                        "shell",
                        fig_util::get_parent_process_exe()
                            .map_or_else(|| "<unknown>".into(), |path| path.display().to_string()),
                    ),
                    (
                        "terminal",
                        fig_util::Terminal::parent_terminal()
                            .map_or_else(|| "<unknown>".into(), |terminal| terminal.internal_id()),
                    ),
                    ("cli_version", env!("CARGO_PKG_VERSION").into())
                ]),
            )),
        ),
    };

    let cli_join = cli::Cli::parse().execute(env_level);

    let result = match track_join {
        Some(track_join) => tokio::join!(cli_join, track_join).0,
        None => cli_join.await,
    };

    if let Err(err) = result {
        if env_level > LevelFilter::INFO {
            eprintln!("{err:?}");
        } else {
            eprintln!("{err}");
        }
        exit(1);
    }
}
