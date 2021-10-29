package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	// updateCmd.Flags().BoolP("force", "f", false, "Force update")

	rootCmd.AddCommand(updateCmd)
}

var updateCmd = &cobra.Command{
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
