use super::{send_command_to_socket, send_recv_command_to_socket};
use anyhow::Result;
use fig_proto::local;
use fig_proto::local::{
    command, BuildCommand, DebugModeCommand, InputMethodAction, InputMethodCommand,
    OpenUiElementCommand, PromptAccessibilityCommand, QuitCommand, RestartCommand,
    RestartSettingsListenerCommand, UiElement, UpdateCommand,
};

pub async fn restart_settings_listener() -> Result<()> {
    let command = command::Command::RestartSettingsListener(RestartSettingsListenerCommand {});
    send_command_to_socket(command).await
}

pub async fn open_ui_element(element: UiElement) -> Result<()> {
    let command = command::Command::OpenUiElement(OpenUiElementCommand {
        element: element.into(),
    });
    send_command_to_socket(command).await
}

pub async fn run_build_command(build: impl Into<String>) -> Result<()> {
    let command = command::Command::Build(BuildCommand {
        branch: Some(build.into()),
    });
    send_command_to_socket(command).await
}

pub async fn toggle_debug_mode() -> Result<local::CommandResponse> {
    let command = command::Command::DebugMode(DebugModeCommand {
        set_debug_mode: None,
        toggle_debug_mode: Some(true),
    });
    send_recv_command_to_socket(command).await
}

pub async fn set_debug_mode(debug_mode: bool) -> Result<local::CommandResponse> {
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

pub async fn restart_command() -> Result<()> {
    let command = command::Command::Restart(RestartCommand {});
    send_command_to_socket(command).await
}

pub async fn quit_command() -> Result<()> {
    let command = command::Command::Quit(QuitCommand {});
    send_command_to_socket(command).await
}

/*
func ReportWindowCommand(message string) error {
    path := os.Getenv("PATH")
    figEnvVar := os.Getenv("FIG_ENV_VAR")
    term := os.Getenv("TERM")

    cmd := fig_proto.Command{
        Command: &fig_proto.Command_ReportWindow{
            ReportWindow: &fig_proto.ReportWindowCommand{
                Report:    message,
                Path:      path,
                FigEnvVar: figEnvVar,
                Terminal:  term,
            },
        },
    }

    err := SendCommand(&cmd)
    if err != nil {
        return err
    }

    return nil
}

func RunInstallScriptCommand() error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_RunInstallScript{
            RunInstallScript: &fig_proto.RunInstallScriptCommand{},
        },
    }

    if err := SendCommand(&cmd); err != nil {
        return err
    }

    return nil
}

func RunResetCacheCommand() error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_ResetCache{
            ResetCache: &fig_proto.ResetCacheCommand{},
        },
    }

    if err := SendCommand(&cmd); err != nil {
        return err
    }

    return nil
}

func GetDebugModeCommand() (string, error) {
    cmd := fig_proto.Command{
        Command: &fig_proto.Command_DebugMode{
            DebugMode: &fig_proto.DebugModeCommand{},
        },
    }

    response, err := SendRecvCommand(&cmd)
    if err != nil {
        return "", err
    }

    return GetCommandResponseMessage(response)
}

*/
