package fig_ipc

import (
	fig_proto "fig-cli/fig-proto"
	"os"
)

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

func GenerateShellContext(
	pid int64,
	tty string,
	shell string,
	sessionId string,
	integrationVersion string,
) *fig_proto.ShellContext {
	wd, _ := os.Getwd()

	return &fig_proto.ShellContext{
		Pid:                     0,
		Ttys:                    tty,
		Shell:                   shell,
		CurrentWorkingDirectory: wd,
		SessionId:               "",
		IntegrationVersion:      &integrationVersion,
	}
}

func CreateEditBufferHook(sessionId string, integrationVersion string, tty string, pid int, histno int, cursor int, text string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Editbuffer{
			Editbuffer: &fig_proto.EditBuffer{
				Context: GenerateShellContext(
					int64(pid),
					tty,
					"",
					sessionId,
					integrationVersion,
				),
				Text:   text,
				Cursor: int64(cursor),
				Histno: int64(histno),
			},
		},
	}
}

func CreatePromptHook(pid int, tty string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Prompt{
			Prompt: &fig_proto.Prompt{
				Context: GenerateShellContext(
					int64(pid),
					tty,
					"",
					"",
					"",
				),
			},
		},
	}
}
