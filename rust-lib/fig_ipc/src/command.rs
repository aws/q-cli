use std::time::Duration;

use fig_proto::local::{
    self,
    command,
    BuildCommand,
    DebugModeCommand,
    InputMethodAction,
    InputMethodCommand,
    OpenUiElementCommand,
    PromptAccessibilityCommand,
    QuitCommand,
    RestartCommand,
    RestartSettingsListenerCommand,
    UiElement,
    UninstallCommand,
    UpdateCommand,
};
use fig_util::directories;
use system_socket::SystemStream;

use super::{
    connect_timeout,
    recv_message,
    send_message,
};
use crate::Error;

type Result<T, E = crate::Error> = std::result::Result<T, E>;

pub async fn restart_settings_listener() -> Result<()> {
    let command = command::Command::RestartSettingsListener(RestartSettingsListenerCommand {});
    send_command_to_socket(command).await
}

pub async fn open_ui_element(element: UiElement, route: Option<String>) -> Result<()> {
    let command = command::Command::OpenUiElement(OpenUiElementCommand {
        element: element.into(),
        route,
    });
    send_command_to_socket(command).await
}

pub async fn run_build_command(build: impl Into<String>) -> Result<()> {
    let command = command::Command::Build(BuildCommand {
        branch: Some(build.into()),
    });
    send_command_to_socket(command).await
}

pub async fn toggle_debug_mode() -> Result<Option<local::CommandResponse>> {
    let command = command::Command::DebugMode(DebugModeCommand {
        set_debug_mode: None,
        toggle_debug_mode: Some(true),
    });
    send_recv_command_to_socket(command).await
}

pub async fn set_debug_mode(debug_mode: bool) -> Result<Option<local::CommandResponse>> {
    let command = command::Command::DebugMode(DebugModeCommand {
        set_debug_mode: Some(debug_mode),
        toggle_debug_mode: None,
    });
    send_recv_command_to_socket(command).await
}

pub async fn input_method_command(action: InputMethodAction) -> Result<()> {
    let command = command::Command::InputMethod(InputMethodCommand {
        actions: Some(action.into()),
    });
    send_command_to_socket(command).await
}

pub async fn prompt_accessibility_command() -> Result<()> {
    let command = command::Command::PromptAccessibility(PromptAccessibilityCommand {});
    send_command_to_socket(command).await
}

pub async fn update_command(force: bool) -> Result<()> {
    let command = command::Command::Update(UpdateCommand { force });
    send_command_to_socket(command).await
}

pub async fn uninstall_command() -> Result<()> {
    let command = command::Command::Uninstall(UninstallCommand {});
    send_command_to_socket(command).await
}

pub async fn restart_command() -> Result<()> {
    let command = command::Command::Restart(RestartCommand {});
    send_command_to_socket(command).await
}

pub async fn quit_command() -> Result<()> {
    let command = command::Command::Quit(QuitCommand {});
    send_command_to_socket(command).await
}

pub async fn run_install_script_command() -> Result<()> {
    let command = command::Command::RunInstallScript(local::RunInstallScriptCommand {});
    send_command_to_socket(command).await
}

pub async fn send_command(connection: &mut SystemStream, command: local::command::Command) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Command(local::Command {
            id: None,
            no_response: Some(false),
            command: Some(command),
        })),
    };

    Ok(send_message(connection, message).await?)
}

pub async fn send_recv_command(
    connection: &mut SystemStream,
    command: local::command::Command,
) -> Result<Option<local::CommandResponse>> {
    send_command(connection, command).await?;
    Ok(tokio::time::timeout(Duration::from_secs(2), recv_message(connection))
        .await
        .or(Err(Error::Timeout))??)
}

pub async fn send_command_to_socket(command: local::command::Command) -> Result<()> {
    let path = directories::fig_socket_path()?;
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_command(&mut conn, command).await
}

pub async fn send_recv_command_to_socket(command: local::command::Command) -> Result<Option<local::CommandResponse>> {
    let path = directories::fig_socket_path()?;
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_recv_command(&mut conn, command).await
}
