use fig_proto::local::UiElement;

use crate::util::{
    launch_fig,
    LaunchArgs,
};

pub async fn execute() -> eyre::Result<()> {
    launch_fig(LaunchArgs {
        print_running: false,
        print_launching: false,
        wait_for_launch: true,
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
