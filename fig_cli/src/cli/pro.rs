use fig_proto::local::UiElement;
use fig_util::launch_fig;

pub async fn execute() -> eyre::Result<()> {
    launch_fig(true, false).ok();

    if fig_ipc::local::open_ui_element(UiElement::MissionControl, Some("/settings/billing".into()))
        .await
        .is_err()
        && fig_util::open_url("https://fig.io/pricing").is_err()
    {
        eyre::bail!("Failed to open https://fig.io/pricing");
    }

    Ok(())
}
