package cmd

import (
	"strconv"

	fig_ipc "fig-cli/fig-ipc"

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
	Use:   "editbuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
	Short: "Run the editbuffer hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 7 {
			return
		}

		pid, _ := strconv.Atoi(args[3])
		histno, _ := strconv.Atoi(args[4])
		cursor, _ := strconv.Atoi(args[5])

		hook := fig_ipc.CreateEditBufferHook(args[0], args[1], args[2], pid, histno, cursor, args[6])
		fig_ipc.SendHook(hook)
		// TODO: Add error handling
	},
}

var hookPromptCmd = &cobra.Command{
	Use:   "prompt [pid] [tty]",
	Short: "Run the prompt hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook := fig_ipc.CreatePromptHook(pid, args[1])
		fig_ipc.SendHook(hook)
		// TODO: Add error handling
	},
}
