use std::{io, time::Duration};

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

/*
func GetCommandResponseMessage(commandResponse *fig_proto.CommandResponse) (string, error) {
    switch commandResponse.Response.(type) {
    case *fig_proto.CommandResponse_Success:
        return commandResponse.GetSuccess().GetMessage(), nil
    case *fig_proto.CommandResponse_Error:
        return commandResponse.GetError().GetMessage(), nil
    default:
        return "", fmt.Errorf("unknown response %T", commandResponse.Response)
    }
}

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

func RestartSettingsListenerCommand() error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_RestartSettingsListener{
            RestartSettingsListener: &fig_proto.RestartSettingsListenerCommand{},
        },
    }

    if err := SendCommand(&cmd); err != nil {
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

func RunBuildCommand(branch string) error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_Build{
            Build: &fig_proto.BuildCommand{
                Branch: &branch,
            },
        },
    }

    if err := SendCommand(&cmd); err != nil {
        return err
    }

    return nil
}

func RunOpenUiElementCommand(element fig_proto.UiElement) (string, error) {
    cmd := fig_proto.Command{
        Command: &fig_proto.Command_OpenUiElement{
            OpenUiElement: &fig_proto.OpenUiElementCommand{
                Element: element,
            },
        },
    }

    response, err := SendRecvCommand(&cmd)
    if err != nil {
        return "", err
    }

    return GetCommandResponseMessage(response)
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

func ToggleDebugModeCommand() (string, error) {
    toggle := true

    cmd := fig_proto.Command{
        Command: &fig_proto.Command_DebugMode{
            DebugMode: &fig_proto.DebugModeCommand{
                ToggleDebugMode: &toggle,
            },
        },
    }

    response, err := SendRecvCommand(&cmd)
    if err != nil {
        return "", err
    }

    return GetCommandResponseMessage(response)
}

func SetDebugModeCommand(debugMode bool) (string, error) {
    cmd := fig_proto.Command{
        Command: &fig_proto.Command_DebugMode{
            DebugMode: &fig_proto.DebugModeCommand{
                SetDebugMode: &debugMode,
            },
        },
    }

    response, err := SendRecvCommand(&cmd)
    if err != nil {
        return "", err
    }

    return GetCommandResponseMessage(response)
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

func PromptAccessibilityCommand() error {
    noResponse := true

    cmd := fig_proto.Command{
        NoResponse: &noResponse,
        Command: &fig_proto.Command_PromptAccessibility{
            PromptAccessibility: &fig_proto.PromptAccessibilityCommand{},
        },
    }

    if err := SendCommand(&cmd); err != nil {
        return err
    }

    return nil
}

func CreateInputMethodRequest(
    action fig_proto.InputMethodAction,
) *fig_proto.Command {
    id := int64(0)
    noResponse := false

    return &fig_proto.Command{
        Id:         &id,
        NoResponse: &noResponse,
        Command: &fig_proto.Command_InputMethod{
            InputMethod: &fig_proto.InputMethodCommand{
                Actions: &action,
            },
        },
    }
}

func InputMethodCommand(command string) (string, error) {
    cmd := CreateInputMethodRequest(fig_proto.InputMethodAction_STATUS_OF_INPUT_METHOD)

    switch command {
    case "install":
        cmd = CreateInputMethodRequest(fig_proto.InputMethodAction_INSTALL_INPUT_METHOD)
    case "uninstall":
        cmd = CreateInputMethodRequest(fig_proto.InputMethodAction_UNINSTALL_INPUT_METHOD)
    case "enable":
        cmd = CreateInputMethodRequest(fig_proto.InputMethodAction_ENABLE_INPUT_METHOD)
    case "disable":
        cmd = CreateInputMethodRequest(fig_proto.InputMethodAction_DISABLE_INPUT_METHOD)
    case "select":
        cmd = CreateInputMethodRequest(fig_proto.InputMethodAction_SELECT_INPUT_METHOD)
    case "deselect":
        cmd = CreateInputMethodRequest(fig_proto.InputMethodAction_DESELECT_INPUT_METHOD)
    case "register":
        cmd = CreateInputMethodRequest(fig_proto.InputMethodAction_REGISTER_INPUT_METHOD)
    default:
        break
    }

    res, err := SendRecvCommand(cmd)
    if err != nil {
        return "", err
    }

    return GetCommandResponseMessage(res)

}
*/
