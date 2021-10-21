package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fmt"

	"github.com/spf13/cobra"
	"google.golang.org/protobuf/proto"
)

func init() {
	rootCmd.AddCommand(testCmd)
}

var testCmd = &cobra.Command{
	Use:   "test",
	Short: "test unix sockets",
	Run: func(cmd *cobra.Command, arg []string) {
		conn, err := fig_ipc.Connect()
		if err != nil {
			fmt.Println(err)
			return
		}

		var id int64 = 0
		res := false
		identifier := "com.googlecode.iterm2"
		action := fig_proto.IntegrationAction_VERIFY_INSTALL
		command := fig_proto.Command{
			Id:         &id,
			NoResponse: &res,
			Command: &fig_proto.Command_TerminalIntegrationUpdate{
				TerminalIntegrationUpdate: &fig_proto.TerminalIntegrationRequest{
					Identifier: identifier,
					Action:     action,
				},
			},
		}

		message := fig_proto.LocalMessage{
			Type: &fig_proto.LocalMessage_Command{
				Command: &command,
			},
		}

		conn.SendFigProto(&message)

		buff, _, err := conn.RecvMessage()
		if err != nil {
			fmt.Println(err)
			return
		}

		var cmdResponse fig_proto.CommandResponse
		proto.Unmarshal(buff, &cmdResponse)

		switch res := cmdResponse.Response.(type) {
		case *fig_proto.CommandResponse_Error:
			fmt.Println("Failure")
			fmt.Println(res.Error.GetExitCode())
			fmt.Println(res.Error.GetMessage())
		case *fig_proto.CommandResponse_Success:
			fmt.Println("Success")
			fmt.Println(res.Success.GetMessage())
		}

	},
}
