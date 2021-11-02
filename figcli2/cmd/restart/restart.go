package restart

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdRestart() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "restart",
		Short: "Restart Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			if err := fig_ipc.RestartCommand(); err != nil {
				fmt.Println("Error:", err)
			}
		},
	}

	return cmd
}
