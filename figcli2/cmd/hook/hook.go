package hook

import (
	"fig-cli/cmd/hook/editbuffer"
	"fig-cli/cmd/hook/event"
	"fig-cli/cmd/hook/hide"
	"fig-cli/cmd/hook/inith"
	integrationready "fig-cli/cmd/hook/integration-ready"
	keyboardfocuschanged "fig-cli/cmd/hook/keyboard-focus-changed"
	preexec "fig-cli/cmd/hook/pre-exec"
	"fig-cli/cmd/hook/prompt"

	"github.com/spf13/cobra"
)

func NewCmdHook() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "hook",
		Short:  "Hook commands",
		Hidden: true,
	}

	cmd.AddCommand(editbuffer.NewCmdEditbuffer())
	cmd.AddCommand(event.NewCmdEvent())
	cmd.AddCommand(hide.NewCmdHide())
	cmd.AddCommand(inith.NewCmdInit())
	cmd.AddCommand(integrationready.NewCmdIntegrationReady())
	cmd.AddCommand(keyboardfocuschanged.NewCmdKeyboardFocusChanged())
	cmd.AddCommand(preexec.NewCmdPreExec())
	cmd.AddCommand(prompt.NewCmdPrompt())

	return cmd
}
