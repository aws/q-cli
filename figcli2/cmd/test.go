package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"
	"time"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(testCmd)
}

var testCmd = &cobra.Command{
	Use:    "test",
	Short:  "test unix sockets",
	Hidden: true,
	Run: func(cmd *cobra.Command, arg []string) {
		conn, err := fig_ipc.Connect()
		if err != nil {
			fmt.Println(err)
			return
		}

		// var id int64 = 0
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

		// conn.SendFigProto(&message)

		msg := conn.RecvMessageTimeout(time.Second * 2)
		if msg.Error != nil {
			fmt.Println(msg.Error)
			return
		}
	},
}
