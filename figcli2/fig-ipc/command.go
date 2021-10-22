package fig_ipc

import (
	fig_proto "fig-cli/fig-proto"
	"fmt"

	"google.golang.org/protobuf/proto"
)

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

	buff, buffType, err := conn.RecvMessage()
	if err != nil {
		return nil, err
	}

	if buffType != protoTypeFigProto {
		return nil, fmt.Errorf("unexpected message type: %d", buffType)
	}

	var cmdResponse fig_proto.CommandResponse
	proto.Unmarshal(buff, &cmdResponse)
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
