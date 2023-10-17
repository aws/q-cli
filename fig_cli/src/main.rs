pub mod cli;
pub mod util;

use std::io::{
    stderr,
    Write,
};
use std::process::exit;

use clap::error::ContextKind;
use clap::Parser;
use eyre::Result;
use fig_log::get_max_fig_log_level;
// use fig_telemetry::sentry::{
//     configure_scope,
//     release_name,
// };
use fig_util::CODEWHISPERER_CLI_BINARY_NAME;
use owo_colors::OwoColorize;
use tracing::metadata::LevelFilter;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();

    // Hyper optimized command parsing for commands we need to be **fast**
    //
    // I literally couldnt make it any faster than this
    if args.get(1).map(String::as_str) == Some("_") {
        match args.get(2).map(String::as_str) {
            Some("get-shell") => {
                cli::internal::get_shell();
                std::process::exit(0);
            },
            Some("should-figterm-launch") => cli::internal::should_figterm_launch::should_figterm_launch(),
            _ => {},
        }
    } else {
        color_eyre::install()?;
    }

    let multithread = matches!(
        args.get(1).map(String::as_str),
        Some("init" | "_" | "internal" | "tips" | "completion" | "hook" | "bg:tmux" | "app:running")
    );

    let parsed = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(err)
            if matches!(
                err.kind(),
                clap::error::ErrorKind::UnknownArgument | clap::error::ErrorKind::InvalidSubcommand
            ) && !err.context().any(|(context_kind, _)| {
                matches!(
                    context_kind,
                    ContextKind::SuggestedSubcommand | ContextKind::SuggestedArg
                )
            }) =>
        {
            err.print()?;
            writeln!(
                stderr(),
                "\nThis command may be valid in newer versions of the CodeWhisperer CLI. Try running {} {}.",
                CODEWHISPERER_CLI_BINARY_NAME.magenta(),
                "update".magenta()
            )
            .ok();
            exit(2);
        },
        Err(err) => {
            err.exit();
        },
    };

    let runtime = if multithread {
        tokio::runtime::Builder::new_multi_thread()
    } else {
        tokio::runtime::Builder::new_current_thread()
    }
    .enable_all()
    .build()?;

    let result = runtime.block_on(parsed.execute());

    if let Err(err) = result {
        if get_max_fig_log_level() > LevelFilter::INFO {
            writeln!(stderr(), "{} {err:?}", "error:".bold().red()).ok();
        } else {
            writeln!(stderr(), "{} {err}", "error:".bold().red()).ok();
        }
        exit(1);
    }

    Ok(())
}
