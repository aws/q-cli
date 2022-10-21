use eyre::Result;
use fig_install::UpdateStatus;

pub async fn update(no_confirm: bool) -> Result<()> {
    match fig_install::update(
        no_confirm,
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
        true,
    )
    .await
    {
        Err(e) => Err(eyre::eyre!(
            "{e}. If this is unexpected, try running `fig doctor` and then try again."
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
