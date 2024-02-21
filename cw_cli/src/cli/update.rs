use eyre::Result;
use fig_install::{
    UpdateOptions,
    UpdateStatus,
};

pub async fn update(non_interactive: bool, relaunch_dashboard: bool, rollout: bool) -> Result<()> {
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
            relaunch_dashboard,
        },
    )
    .await;

    match res {
        Err(e) => Err(eyre::eyre!(
            "{e}. If this is unexpected, try running `cw doctor` and then try again."
        )),
        Ok(false) => {
            println!(
                "No updates available, \n{} is the latest version.",
                env!("CARGO_PKG_VERSION")
            );
            Ok(())
        },
        Ok(true) => Ok(()),
    }
}
