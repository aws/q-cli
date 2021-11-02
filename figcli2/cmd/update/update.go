package update

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

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
				panic(err)
			}
		},
	}

	return cmd
}
