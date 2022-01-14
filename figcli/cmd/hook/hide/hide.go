package hide

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"

	"github.com/spf13/cobra"
)

func NewCmdHide() *cobra.Command {
	cmd := &cobra.Command{
		Use:                "hide",
		Short:              "Run the hide hook",
		DisableFlagParsing: true,
		Run: func(cmd *cobra.Command, args []string) {
			hook := fig_ipc.CreateHideHook()
			err := fig_ipc.SendHook(hook)
			if err != nil {
				logging.Log("Error:", err.Error())
				return
			}
		},
	}

	return cmd
}
