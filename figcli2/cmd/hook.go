package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"
	"strconv"

	"github.com/spf13/cobra"
)

func init() {
	hookCmd.AddCommand(hookEditbufferCmd)
	hookCmd.AddCommand(hookPromptCmd)

	rootCmd.AddCommand(hookCmd)
}

var hookCmd = &cobra.Command{
	Use:    "hook",
	Short:  "Run a hook",
	Hidden: true,
}

var hookEditbufferCmd = &cobra.Command{
	Use:   "editbuffer [text] [cursor] [shell] [session-id]",
	Short: "Run the editbuffer hook",
	Args:  cobra.ExactArgs(4),
	Run: func(cmd *cobra.Command, args []string) {
		cursor, err := strconv.ParseInt(args[1], 10, 64)
		if err != nil {
			fmt.Println("Error", err)
			return
		}

		hook := fig_ipc.CreateEditBufferHook(args[0], cursor, args[2], args[3])
		if err := fig_ipc.SendHook(hook); err != nil {
			fmt.Println("Error:", err)
		} else {
			fmt.Println("Success")
		}
	},
}

var hookPromptCmd = &cobra.Command{
	Use:   "prompt [pid] [shell] [current-working-directory] [session-id]",
	Short: "Run the prompt hook",
	Args:  cobra.ExactArgs(4),
	Run: func(cmd *cobra.Command, args []string) {
		pid64, err := strconv.ParseInt(args[0], 10, 32)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		pid := int32(pid64)

		hook := fig_ipc.CreatePromptHook(pid, args[1], args[2], args[3])
		if err := fig_ipc.SendHook(hook); err != nil {
			fmt.Println("Error:", err)
		} else {
			fmt.Println("Success")
		}
	},
}
