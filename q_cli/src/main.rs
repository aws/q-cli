pub mod cli;
pub mod util;

use std::io::{
    stderr,
    Write,
};
use std::process::ExitCode;

use clap::error::{
    ContextKind,
    ErrorKind,
};
use clap::Parser;
use eyre::Result;
use fig_log::get_max_fig_log_level;
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};
use owo_colors::OwoColorize;
use tracing::metadata::LevelFilter;

fn main() -> Result<ExitCode> {
    color_eyre::install()?;

    let multithread = matches!(
        std::env::args().nth(1).as_deref(),
        Some("init" | "_" | "internal" | "completion" | "hook")
    );

    let parsed = match cli::Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();

            let unknown_arg = matches!(err.kind(), ErrorKind::UnknownArgument | ErrorKind::InvalidSubcommand)
                && !err.context().any(|(context_kind, _)| {
                    matches!(
                        context_kind,
                        ContextKind::SuggestedSubcommand | ContextKind::SuggestedArg
                    )
                });

            if unknown_arg {
                let _ = writeln!(
                    stderr(),
                    "\nThis command may be valid in newer versions of the {PRODUCT_NAME} CLI. Try running {} {}.",
                    CLI_BINARY_NAME.magenta(),
                    "update".magenta()
                );
            }

            return Ok(ExitCode::from(err.exit_code().try_into().unwrap_or(2)));
        },
    };

    let verbose = parsed.verbose > 0;

    let runtime = if multithread {
        tokio::runtime::Builder::new_multi_thread()
    } else {
        tokio::runtime::Builder::new_current_thread()
    }
    .enable_all()
    .build()?;

    let result = runtime.block_on(async {
        let result = parsed.execute().await;
        fig_telemetry::finish_telemetry().await;
        result
    });

    match result {
        Ok(exit_code) => Ok(exit_code),
        Err(err) => {
            if verbose || get_max_fig_log_level() > LevelFilter::INFO {
                let _ = writeln!(stderr(), "{} {err:?}", "error:".bold().red());
            } else {
                let _ = writeln!(stderr(), "{} {err}", "error:".bold().red());
            }
            Ok(ExitCode::FAILURE)
        },
    }
}
