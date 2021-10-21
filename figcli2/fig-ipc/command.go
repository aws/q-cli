package fig_ipc

import (
	fig_proto "fig-cli/fig-proto"
	"fmt"

	"google.golang.org/protobuf/proto"
)

func SendRecvCommand(cmd *fig_proto.Command) (interface{}, error) {
	conn, err := Connect()
	if err != nil {
		return "", err
	}
	defer conn.Close()

	message := fig_proto.LocalMessage{
		Type: &fig_proto.LocalMessage_Command{
			Command: cmd,
		},
	}

	if err = conn.SendFigProto(&message); err != nil {
		return "", err
	}

	buff, buffType, err := conn.RecvMessage()
	if err != nil {
		return "", err
	}

	if buffType != protoTypeFigProto {
		return "", fmt.Errorf("unexpected message type: %d", buffType)
	}

	var cmdResponse fig_proto.CommandResponse
	proto.Unmarshal(buff, &cmdResponse)

	switch res := cmdResponse.Response.(type) {
	case *fig_proto.CommandResponse_Error:
		return nil, fmt.Errorf("%s", res.Error.GetMessage())
	case *fig_proto.CommandResponse_Success:
		return res.Success.GetMessage(), nil
	case *fig_proto.CommandResponse_IntegrationList:
		return res.IntegrationList.GetIntegrations(), nil
	default:
		return "", fmt.Errorf("unexpected response type: %T", res)
	}
}
