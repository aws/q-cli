package initp

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdInit() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "init",
		Short: "Reload the settings listener",
		Run: func(cmd *cobra.Command, arg []string) {
			err := fig_ipc.RestartSettingsListenerCommand()
			if err != nil {
				fmt.Println(err)
				return
			}

			fmt.Printf("\nSettings listener restarted.\n\n")
		},
	}

	return cmd
}
