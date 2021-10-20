package cmd

import (
	"fmt"
	"fig-cli/fig-ipc"
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

		result.SendFigJson("{ \"command\": { \"id\": 1, \"terminalIntegrationUpdate\": { \"identifier\": \"com.googlecode.iterm2\", \"action\": 1 } } }")

	},
}
