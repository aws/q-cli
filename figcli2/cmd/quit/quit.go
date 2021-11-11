package quit

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdQuit() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "quit",
		Short: "Quit Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Println("→ Quitting Fig...")
			if err := fig_ipc.QuitCommand(); err != nil {
				fmt.Println("Error:", err)
			}
		},
	}

	return cmd
}
