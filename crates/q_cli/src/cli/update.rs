use std::process::ExitCode;

use anstream::println;
use clap::Args;
use crossterm::style::Stylize;
use eyre::Result;
use fig_install::{
    UpdateOptions,
    UpdateStatus,
};
use fig_os_shim::{
    Context,
    Os,
    PlatformProvider,
};
use fig_util::CLI_BINARY_NAME;
use fig_util::manifest::{
    Variant,
    manifest,
};

#[derive(Debug, PartialEq, Args)]
pub struct UpdateArgs {
    /// Don't prompt for confirmation
    #[arg(long, short = 'y')]
    non_interactive: bool,
    /// Relaunch into dashboard after update (false will launch in background)
    #[arg(long, default_value = "true")]
    relaunch_dashboard: bool,
    /// Uses rollout
    #[arg(long)]
    rollout: bool,
}

impl UpdateArgs {
    pub async fn execute(&self) -> Result<ExitCode> {
        let ctx = Context::new();
        if ctx.platform().os() == Os::Linux && manifest().variant == Variant::Full {
            match fig_install::check_for_updates(true).await {
                Ok(Some(pkg)) => {
                    println!("A new version of {} is available: {}", CLI_BINARY_NAME, pkg.version);
                    return Ok(ExitCode::SUCCESS);
                },
                Ok(None) => {
                    println!(
                        "No updates available, \n{} is the latest version.",
                        env!("CARGO_PKG_VERSION").bold()
                    );
                    return Ok(ExitCode::SUCCESS);
                },
                Err(err) => {
                    eyre::bail!(
                        "{err}\n\nFailed checking for updates. If this is unexpected, try running {} and then try again.\n",
                        format!("{CLI_BINARY_NAME} doctor").bold()
                    )
                },
            }
        }

        let UpdateArgs {
            non_interactive,
            relaunch_dashboard,
            rollout,
        } = &self;

        let res = fig_install::update(
            Context::new(),
            Some(Box::new(|mut recv| {
                tokio::runtime::Handle::current().spawn(async move {
                    let progress_bar = indicatif::ProgressBar::new(100);
                    loop {
                        match recv.recv().await {
                            Some(UpdateStatus::Percent(p)) => {
                                progress_bar.set_position(p as u64);
                            },
                            Some(UpdateStatus::Message(m)) => {
                                progress_bar.set_message(m);
                            },
                            Some(UpdateStatus::Error(e)) => {
                                progress_bar.abandon();
                                return Err(eyre::eyre!(e));
                            },
                            Some(UpdateStatus::Exit) | None => {
                                progress_bar.finish_with_message("Done!");
                                break;
                            },
                        }
                    }
                    Ok(())
                });
            })),
            UpdateOptions {
                ignore_rollout: !rollout,
                interactive: !non_interactive,
                relaunch_dashboard: *relaunch_dashboard,
            },
        )
        .await;

        match res {
            Ok(true) => Ok(ExitCode::SUCCESS),
            Ok(false) => {
                println!(
                    "No updates available, \n{} is the latest version.",
                    env!("CARGO_PKG_VERSION").bold()
                );
                Ok(ExitCode::SUCCESS)
            },
            Err(err) => eyre::bail!(
                "{err}\n\nIf this is unexpected, try running {} and then try again.\n",
                format!("{CLI_BINARY_NAME} doctor").bold()
            ),
        }
    }
}
