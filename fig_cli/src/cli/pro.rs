use fig_proto::local::UiElement;

pub async fn execute() -> eyre::Result<()> {
    crate::util::launch_fig(crate::util::LaunchOptions {
        wait_for_activation: true,
        verbose: false,
    })
    .ok();

    if fig_ipc::local::open_ui_element(UiElement::MissionControl, Some("/settings/billing".into()))
        .await
        .is_err()
        && fig_util::open_url("https://fig.io/pricing").is_err()
    {
        eyre::bail!("Failed to open https://fig.io/pricing");
    }

    Ok(())
}
