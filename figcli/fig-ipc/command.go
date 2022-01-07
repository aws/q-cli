package fig_ipc

import (
	fig_proto "fig-cli/fig-proto"
	"fmt"
	"os"
	"time"

	"google.golang.org/protobuf/proto"
)

func SendCommand(cmd *fig_proto.Command) error {
	conn, err := Connect()
	if err != nil {
		return err
	}
	defer conn.Close()

	message := fig_proto.LocalMessage{
		Type: &fig_proto.LocalMessage_Command{
			Command: cmd,
		},
	}

	if err := conn.SendFigProto(&message); err != nil {
		return err
	}

	return nil
}

func SendRecvCommand(cmd *fig_proto.Command) (*fig_proto.CommandResponse, error) {
	conn, err := Connect()
	if err != nil {
		return nil, err
	}
	defer conn.Close()

	message := fig_proto.LocalMessage{
		Type: &fig_proto.LocalMessage_Command{
			Command: cmd,
		},
	}

	if err = conn.SendFigProto(&message); err != nil {
		return nil, err
	}

	msg := conn.RecvMessageTimeout(time.Second * 3)
	if msg.Error != nil {
		return nil, msg.Error
	}

	if msg.ProtoType != protoTypeFigProto {
		return nil, fmt.Errorf("unexpected message type: %d", msg.ProtoType)
	}

	var cmdResponse fig_proto.CommandResponse
	proto.Unmarshal(msg.Message, &cmdResponse)
	return &cmdResponse, nil
}

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
	default:
		break
	}

	res, err := SendRecvCommand(cmd)
	if err != nil {
		return "", err
	}

	return GetCommandResponseMessage(res)

}
