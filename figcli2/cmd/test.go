package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(testCmd)
}

var testCmd = &cobra.Command{
	Use:   "test",
	Short: "test unix sockets",
	Run: func(cmd *cobra.Command, arg []string) {
		result, err := fig_ipc.Connect()
		if err != nil {
			fmt.Println(err)
			return
		}

		// var id int64 = 1
		// res := false
		// identifier := "com.googlecode.iterm2"
		// action := fig_proto.IntegrationAction_VERIFY_INSTALL
		// command := fig_proto.Command{
		// 	Id:         &id,
		// 	NoResponse: &res,
		// 	Command: &fig_proto.Command_TerminalIntegrationUpdate{
		// 		TerminalIntegrationUpdate: &fig_proto.TerminalIntegrationRequest{
		// 			Identifier: identifier,
		// 			Action:     action,
		// 		},
		// 	},
		// }

		// message := fig_proto.LocalMessage{
		// 	Type: &fig_proto.LocalMessage_Command{
		// 		Command: &command,
		// 	},
		// }

		// result.SendFigProto(&message)
		result.SendFigJson("{ \"command\": { \"id\": 1, \"terminalIntegrationUpdate\": { \"identifier\": \"com.googlecode.iterm2\", \"action\": 1 } } }")

	},
}
