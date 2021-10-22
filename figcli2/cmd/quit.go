package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(quitCmd)
}

var quitCmd = &cobra.Command{
	Use:   "quit",
	Short: "Quit Fig",
	Run: func(cmd *cobra.Command, arg []string) {
		if err := fig_ipc.QuitCommand(); err != nil {
			fmt.Println("Error:", err)
		}
	},
}
