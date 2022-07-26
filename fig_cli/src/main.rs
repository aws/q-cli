#[macro_use]
extern crate cfg_if;

pub mod cli;
pub mod daemon;
pub mod util;

use std::io::{
    stderr,
    Write,
};
use std::process::exit;
use std::str::FromStr;

use clap::StructOpt;
use fig_telemetry::sentry::{
    configure_scope,
    release_name,
};
use tracing::level_filters::LevelFilter;

const SENTRY_CLI_URL: &str = "https://0631fceb9ae540bb874af81820507ebf@o436453.ingest.sentry.io/6187837";

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
        Some("daemon") => (Some(fig_telemetry::init_sentry(release_name!(), SENTRY_CLI_URL)), None),
        _ => {
            let sentry = fig_telemetry::init_sentry(release_name!(), SENTRY_CLI_URL);

            let arguments = std::env::args().collect::<Vec<_>>().join(" ");
            let shell = fig_util::get_parent_process_exe()
                .map_or_else(|| "<unknown>".into(), |path| path.display().to_string());
            let terminal = fig_util::Terminal::parent_terminal()
                .map_or_else(|| "<unknown>".into(), |terminal| terminal.internal_id());
            let cli_version = env!("CARGO_PKG_VERSION").into();

            configure_scope(|scope| {
                scope.set_tag("arguments", &arguments);
                scope.set_tag("shell", &shell);
                scope.set_tag("terminal", &terminal);
                scope.set_tag("cli_version", &cli_version);
            });

            (
                Some(sentry),
                Some(fig_telemetry::dispatch_emit_track(
                    fig_telemetry::TrackEvent::new(
                        fig_telemetry::TrackEventType::RanCommand,
                        fig_telemetry::TrackSource::Cli,
                        [
                            ("arguments", arguments),
                            ("shell", shell),
                            ("terminal", terminal),
                            ("cli_version", cli_version),
                        ],
                    ),
                    false,
                )),
            )
        },
    };

    let cli_join = cli::Cli::parse().execute(env_level);

    let result = match track_join {
        Some(track_join) => tokio::join!(cli_join, track_join).0,
        None => cli_join.await,
    };

    if let Err(err) = result {
        if env_level > LevelFilter::INFO {
            writeln!(stderr(), "{err:?}").ok();
        } else {
            writeln!(stderr(), "{err}").ok();
        }
        exit(1);
    }
}
