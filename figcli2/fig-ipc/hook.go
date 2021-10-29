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

	message := fig_proto.LocalMessage{
		Type: &fig_proto.LocalMessage_Hook{
			Hook: hook,
		},
	}

	if err = conn.SendFigProto(&message); err != nil {
		return err
	}

	if err = conn.Close(); err != nil {
		return err
	}

	return nil
}

func GenerateShellContext(
	pid int32,
	tty string,
	sessionId string,
	integrationVersion int32,
) (*fig_proto.ShellContext, error) {
	wd, err := os.Getwd()
	if err != nil {
		return nil, err
	}

	shell, err := GetShell()
	if err != nil {
		return nil, err
	}

	return &fig_proto.ShellContext{
		Pid:                     &pid,
		Ttys:                    &tty,
		ProcessName:             &shell,
		CurrentWorkingDirectory: &wd,
		SessionId:               &sessionId,
		IntegrationVersion:      &integrationVersion,
	}, nil
}

func CreateEditBufferHook(sessionId string, integrationVersion int, tty string, pid int, histno int, cursor int, text string) (*fig_proto.Hook, error) {
	context, err := GenerateShellContext(
		int32(pid),
		tty,
		sessionId,
		int32(integrationVersion),
	)

	if err != nil {
		return nil, err
	}

	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_EditBuffer{
			EditBuffer: &fig_proto.EditBufferHook{
				Context: context,
				Text:    text,
				Cursor:  int64(cursor),
				Histno:  int64(histno),
			},
		},
	}, nil
}

func CreatePromptHook(pid int, tty string) (*fig_proto.Hook, error) {
	context, err := GenerateShellContext(
		int32(pid),
		tty,
		os.Getenv("TERM_SESSION_ID"),
		int32(currentIntegrationVersion),
	)

	if err != nil {
		return nil, err
	}

	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Prompt{
			Prompt: &fig_proto.PromptHook{
				Context: context,
			},
		},
	}, nil
}

func CreateInitHook(pid int, tty string) (*fig_proto.Hook, error) {
	env := os.Environ()
	envMap := make(map[string]string)
	for _, e := range env {
		pair := strings.Split(e, "=")
		envMap[pair[0]] = pair[1]
	}

	term, err := GetCurrentTerminal()
	if err != nil {
		return nil, err
	}

	bundle, err := term.PotentialBundleId()
	if err != nil {
		return nil, err
	}

	context, err := GenerateShellContext(
		int32(pid),
		tty,
		os.Getenv("TERM_SESSION_ID"),
		int32(currentIntegrationVersion),
	)

	if err != nil {
		return nil, err
	}

	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_Init{
			Init: &fig_proto.InitHook{
				Context:      context,
				CalledDirect: false,
				Bundle:       bundle,
				Env:          envMap,
			},
		},
	}, nil
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

func CreatePreExecHook(pid int, tty string) (*fig_proto.Hook, error) {
	context, err := GenerateShellContext(
		int32(pid),
		tty,
		os.Getenv("TERM_SESSION_ID"),
		int32(currentIntegrationVersion),
	)

	if err != nil {
		return nil, err
	}

	return &fig_proto.Hook{
		Hook: &fig_proto.Hook_PreExec{
			PreExec: &fig_proto.PreExecHook{
				Context: context,
				Command: "",
			},
		},
	}, nil
}
