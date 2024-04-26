use clap::Args;
use crossterm::style::Stylize;
use eyre::Result;
use fig_install::{
    UpdateOptions,
    UpdateStatus,
};
use fig_util::CLI_BINARY_NAME;

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
    pub async fn execute(&self) -> Result<()> {
        let UpdateArgs {
            non_interactive,
            relaunch_dashboard,
            rollout,
        } = &self;

        let res = fig_install::update(
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
            Ok(true) => Ok(()),
            Ok(false) => {
                println!(
                    "No updates available, \n{} is the latest version.",
                    env!("CARGO_PKG_VERSION").bold()
                );
                Ok(())
            },
            Err(err) => eyre::bail!(
                "{err}\n\nIf this is unexpected, try running {} and then try again.\n",
                format!("{CLI_BINARY_NAME} doctor").bold()
            ),
        }
    }
}
