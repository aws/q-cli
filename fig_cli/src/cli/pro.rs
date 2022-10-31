use fig_proto::local::UiElement;
use fig_util::desktop::{
    launch_fig_desktop,
    LaunchArgs,
};

pub async fn execute() -> eyre::Result<()> {
    launch_fig_desktop(LaunchArgs {
        wait_for_socket: true,
        open_dashboard: false,
        immediate_update: true,
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
