package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(restartCmd)
}

var restartCmd = &cobra.Command{
	Use:   "restart",
	Short: "Restart Fig",
	Run: func(cmd *cobra.Command, arg []string) {
		if err := fig_ipc.RestartCommand(); err != nil {
			fmt.Println("Error:", err)
		}
	},
}
