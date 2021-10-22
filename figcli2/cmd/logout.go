package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(logoutCmd)
}

var logoutCmd = &cobra.Command{
	Use:   "logout",
	Short: "Logout of Fig",
	Run: func(cmd *cobra.Command, arg []string) {
		// Logout
		logout := fig_proto.Command{
			Command: &fig_proto.Command_Logout{},
		}

		res, err := fig_ipc.SendRecvCommand(&logout)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		message, err := fig_ipc.GetCommandResponseMessage(res)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		fmt.Println(message)

		// Restart Fig
		if err := fig_ipc.RestartCommand(); err != nil {
			fmt.Println("Error:", err)
			return
		}
	},
}
