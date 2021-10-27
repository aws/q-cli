package fig_ipc

import (
	fig_proto "fig-cli/fig-proto"
	"os"
	"strings"
)

const (
	currentIntegrationVersion = 5
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
	pid int32,
	tty string,
	sessionId string,
	integrationVersion int32,
) *fig_proto.ShellContext {
	wd, _ := os.Getwd()
	shell, _ := GetShell()

	return &fig_proto.ShellContext{
		Pid:                     &pid,
		Ttys:                    &tty,
		ProcessName:             &shell,
		CurrentWorkingDirectory: &wd,
		SessionId:               &sessionId,
		IntegrationVersion:      &integrationVersion,
	}
}

func CreateEditBufferHook(sessionId string, integrationVersion int, tty string, pid int, histno int, cursor int, text string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_EditBuffer{
			EditBuffer: &fig_proto.EditBufferHook{
				Context: GenerateShellContext(
					int32(pid),
					tty,
					sessionId,
					int32(integrationVersion),
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
			Prompt: &fig_proto.PromptHook{
				Context: GenerateShellContext(
					int32(pid),
					tty,
					os.Getenv("TERM_SESSION_ID"),
					int32(currentIntegrationVersion),
				),
			},
		},
	}
}

func CreateInitHook(pid int, tty string) *fig_proto.Hook {
	env := os.Environ()
	envMap := make(map[string]string)
	for _, e := range env {
		pair := strings.Split(e, "=")
		envMap[pair[0]] = pair[1]
	}

	term, _ := GetCurrentTerminal()
	bundle, _ := term.PotentialBundleId()

	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Init{
			Init: &fig_proto.InitHook{
				Context: GenerateShellContext(
					int32(pid),
					tty,
					os.Getenv("TERM_SESSION_ID"),
					int32(currentIntegrationVersion),
				),
				CalledDirect: false,
				Bundle:       bundle,
				Env:          envMap,
			},
		},
	}
}

func CreateKeyboardFocusChangedHook(bundleIdentifier string, focusedSessionId string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_KeyboardFocusChanged{
			KeyboardFocusChanged: &fig_proto.KeyboardFocusChangedHook{
				BundleIdentifier: bundleIdentifier,
				FocusedSessionId: focusedSessionId,
			},
		},
	}
}

func CreateIntegrationReadyHook(identifyier string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_IntegrationReady{
			IntegrationReady: &fig_proto.IntegrationReadyHook{
				Identifier: identifyier,
			},
		},
	}
}

func CreateHideHook() *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Hide{
			Hide: &fig_proto.HideHook{},
		},
	}
}

func CreateEventHook(eventName string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Event{
			Event: &fig_proto.EventHook{
				EventName: eventName,
			},
		},
	}
}

func CreatePreExecHook(pid int, tty string) *fig_proto.Hook {
	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_PreExec{
			PreExec: &fig_proto.PreExecHook{
				Context: GenerateShellContext(
					int32(pid),
					tty,
					os.Getenv("TERM_SESSION_ID"),
					int32(currentIntegrationVersion),
				),
				Command: "",
			},
		},
	}
}
