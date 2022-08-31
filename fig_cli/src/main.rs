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
use serde_json::json;
use tracing::metadata::LevelFilter;

const SENTRY_CLI_URL: &str = "https://0631fceb9ae540bb874af81820507ebf@o436453.ingest.sentry.io/6187837";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();

    // Whitelist commands do not have sentry or telemetry, telemetry should only run on
    // user facing commands as performance is less important
    let (_guard, track_join) = match (
        args.get(0).map(String::as_str),
        args.get(1).map(String::as_str),
        args.get(2).map(String::as_str),
    ) {
        (_, Some("init" | "_" | "internal" | "tips" | "completion" | "hook" | "bg:tmux"), _) => (None, None),
        (Some("/Applications/Fig.app/Contents/MacOS/fig-darwin-universal"), _, _)
        | (_, Some("app"), Some("prompts")) => (
            Some(fig_telemetry::init_sentry(release_name!(), SENTRY_CLI_URL, 1.0, false)),
            None,
        ),
        _ => {
            let sentry = fig_telemetry::init_sentry(release_name!(), SENTRY_CLI_URL, 1.0, false);

            let shell = fig_util::get_parent_process_exe()
                .map_or_else(|| "<unknown>".into(), |path| path.display().to_string());
            let terminal = fig_util::Terminal::parent_terminal()
                .map_or_else(|| "<unknown>".into(), |terminal| terminal.internal_id());
            let cli_version = env!("CARGO_PKG_VERSION");

            configure_scope(|scope| {
                scope.set_tag("arguments", &args.join(" "));
                scope.set_tag("shell", &shell);
                scope.set_tag("terminal", &terminal);
                scope.set_tag("cli_version", &cli_version);
            });

            match std::env::var_os("FIG_NO_RAN_COMMAND") {
                None => (
                    Some(sentry),
                    Some(fig_telemetry::dispatch_emit_track(
                        fig_telemetry::TrackEvent::new(
                            fig_telemetry::TrackEventType::RanCommand,
                            fig_telemetry::TrackSource::Cli,
                            [
                                ("arguments", json!(args)),
                                ("shell", json!(shell)),
                                ("terminal", json!(terminal)),
                                ("cli_version", json!(cli_version)),
                                ("arg0", json!(args.get(0))),
                                ("arg1", json!(args.get(1))),
                            ],
                        ),
                        false,
                    )),
                ),
                Some(_) => (Some(sentry), None),
            }
        },
    };

    color_eyre::install()?;

    let parsed = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) if matches!(err.kind, clap::ErrorKind::UnknownArgument) => {
            err.print()?;
            println!(
                "This command may be valid in newer versions of the Fig CLI. Try running {}",
                "fig update".magenta()
            );
            return Ok(());
        },
        Err(err) => {
            err.print()?;
            return Ok(());
        },
    };

    let cli_join = parsed.execute();

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
