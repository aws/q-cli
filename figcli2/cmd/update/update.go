package update

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

func NewCmdUpdate() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "update",
		Short: "Update Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Printf("\n→ Checking for updates to macOS app...\n\n")

			err := fig_ipc.UpdateCommand(false)
			if err != nil {
				logging.Log("fig update:", err.Error())
				fmt.Println("Unable to update fig")
				os.Exit(1)
			}
		},
	}

	return cmd
}
