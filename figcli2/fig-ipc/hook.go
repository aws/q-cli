package fig_ipc

import fig_proto "fig-cli/fig-proto"

func SendHook(hook *fig_proto.Hook) error {
	conn, err := Connect()
	if err != nil {
		return err
	}
	defer conn.Close()

	message := fig_proto.LocalMessage{
		Type: &fig_proto.LocalMessage_Hook{
			Hook: hook,
		},
	}

	if err = conn.SendFigProto(&message); err != nil {
		return err
	}

	return nil
}

func CreateEditBufferHook(text string, cursor int64, shell string, sessionId string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Editbuffer{
			Editbuffer: &fig_proto.EditBuffer{
				Text:      text,
				Cursor:    cursor,
				Shell:     shell,
				SessionId: sessionId,
			},
		},
	}
}

func CreatePromptHook(pid int32, shell string, currentWorkingDirectory string, sesstionId string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Prompt{
			Prompt: &fig_proto.Prompt{
				Pid:                     pid,
				Shell:                   shell,
				CurrentWorkingDirectory: currentWorkingDirectory,
				SessionId:               sesstionId,
			},
		},
	}
}
