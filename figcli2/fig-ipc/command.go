package fig_ipc

import (
	fig_proto "fig-cli/fig-proto"
	"fmt"
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
