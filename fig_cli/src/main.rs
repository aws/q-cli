pub mod cli;
pub mod daemon;
pub mod util;

use std::io::{
    stderr,
    Write,
};
use std::process::exit;

use clap::StructOpt;
use crossterm::style::Stylize;
use eyre::Result;
use fig_log::FIG_LOG_LEVEL;
use fig_telemetry::sentry::{
    configure_scope,
    release_name,
};
use tracing::metadata::LevelFilter;

const SENTRY_CLI_URL: &str = "https://0631fceb9ae540bb874af81820507ebf@o436453.ingest.sentry.io/6187837";

#[tokio::main]
async fn main() -> Result<()> {
    // Whitelist commands do not have sentry or telemetry, telemetry should only run on
    // user facing commands as performance is less important
    let (_guard, track_join) = match (std::env::args().nth(1).as_deref(), std::env::args().nth(2).as_deref()) {
        (Some("init" | "_" | "internal" | "tips" | "completion" | "hook"), _) => (None, None),
        (Some("daemon"), _) | (Some("login"), Some("-r")) | (Some("app"), Some("prompt")) => (
            Some(fig_telemetry::init_sentry(release_name!(), SENTRY_CLI_URL, 1.0, false)),
            None,
        ),
        _ => {
            let sentry = fig_telemetry::init_sentry(release_name!(), SENTRY_CLI_URL, 1.0, false);

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

    color_eyre::install()?;

    let cli_join = cli::Cli::parse().execute();

    let result = match track_join {
        Some(track_join) => tokio::join!(cli_join, track_join).0,
        None => cli_join.await,
    };

    if let Err(err) = result {
        if *FIG_LOG_LEVEL > LevelFilter::INFO {
            writeln!(stderr(), "{} {err:?}", "error:".bold().red()).ok();
        } else {
            writeln!(stderr(), "{} {err}", "error:".bold().red()).ok();
        }
        exit(1);
    }

    Ok(())
}
