package keyboardfocuschanged

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"

	"github.com/spf13/cobra"
)

func NewCmdKeyboardFocusChanged(hidden bool) *cobra.Command {
	cmd := &cobra.Command{
		Use:                "keyboard-focus-changed [app-identifier] [focused-session-id]",
		Short:              "Run the keyboard-focus-changed hook",
		DisableFlagParsing: true,
		Hidden:             hidden,
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) != 2 {
				return
			}

			hook := fig_ipc.CreateKeyboardFocusChangedHook(args[0], args[1])
			err := fig_ipc.SendHook(hook)
			if err != nil {
				logging.Log("Error:", err.Error())
				return
			}
		},
	}

	return cmd
}
