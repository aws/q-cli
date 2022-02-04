use std::{io, time::Duration};

use super::{connect_timeout, get_socket_path};
use crate::proto::local::{
    command, BuildCommand, DebugModeCommand, InputMethodAction, InputMethodCommand,
    OpenUiElementCommand, PromptAccessibilityCommand, RestartSettingsListenerCommand, UiElement,
};
use crate::proto::{local, FigProtobufEncodable};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use prost::Message;
use tokio::{io::AsyncWriteExt, net::UnixStream};

pub async fn send_command(
    connection: &mut UnixStream,
    command: local::command::Command,
) -> Result<()> {
    let message = local::LocalMessage {
        r#type: Some(local::local_message::Type::Command(local::Command {
            id: None,
            no_response: Some(false),
            command: Some(command),
        })),
    };

    let encoded_message = message.encode_fig_protobuf()?;

    connection.write_all(&encoded_message).await?;
    Ok(())
}

pub async fn send_recv_command(
    connection: &mut UnixStream,
    command: local::command::Command,
) -> Result<local::CommandResponse> {
    send_command(connection, command).await?;

    tokio::time::timeout(Duration::from_secs(2), connection.readable()).await??;
    let mut proto_type: [u8; 10] = [0; 10];
    let proto_type = match connection.try_read(&mut proto_type) {
        Ok(10) => std::str::from_utf8(&proto_type)?,
        Ok(n) => anyhow::bail!("Read {} bytes for message type", n),
        Err(e) => anyhow::bail!("Could not get message type {}", e),
    };

    let mut msg_size: [u8; 8] = [0; 8];
    connection.readable().await?;
    let msg_size = match connection.try_read(&mut msg_size) {
        Ok(8) => u64::from_be_bytes(msg_size),
        Ok(n) => anyhow::bail!("Read {} bytes for message size", n),
        Err(e) => anyhow::bail!("Could not get message size {}", e),
    };

    let mut buf = BytesMut::new();
    let mut bytes_left: usize = usize::try_from(msg_size)?;
    loop {
        connection.readable().await?;
        match connection.try_read_buf(&mut buf) {
            Ok(n) => {
                if bytes_left <= n || n == 0 {
                    break;
                }
                bytes_left -= n;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => anyhow::bail!(e),
        }
    }

    if proto_type != "\x1b@fig-pbuf" {
        anyhow::bail!("Unexpected message type");
    }

    local::CommandResponse::decode(buf.as_ref()).map_err(|err| anyhow!(err))
}

pub async fn send_command_to_socket(command: local::command::Command) -> Result<()> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_command(&mut conn, command).await
}

pub async fn send_recv_command_to_socket(
    command: local::command::Command,
) -> Result<local::CommandResponse> {
    let path = get_socket_path();
    let mut conn = connect_timeout(&path, Duration::from_secs(3)).await?;
    send_recv_command(&mut conn, command).await
}

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

/*
func RestartCommand() error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_Restart{
            Restart: &fig_proto.RestartCommand{},
        },
    }

    if err := SendCommand(&cmd); err != nil {
        return err
    }

    return nil
}

func QuitCommand() error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_Quit{
            Quit: &fig_proto.QuitCommand{},
        },
    }

    if err := SendCommand(&cmd); err != nil {
        return err
    }

    return nil
}

func UpdateCommand(force bool) error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_Update{
            Update: &fig_proto.UpdateCommand{
                Force: force,
            },
        },
    }

    if err := SendCommand(&cmd); err != nil {
        return err
    }

    return nil
}

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
