package cmd

import (
	"strconv"

	fig_ipc "fig-cli/fig-ipc"

	"github.com/spf13/cobra"
)

func init() {
	hookCmd.AddCommand(hookEditbufferCmd)
	hookCmd.AddCommand(hookPromptCmd)
	hookCmd.AddCommand(hookInitCmd)
	hookCmd.AddCommand(hookKeyboardFocusChanged)
	hookCmd.AddCommand(hookIntegrationReady)

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

		integrationVersion, _ := strconv.Atoi(args[1])
		pid, _ := strconv.Atoi(args[3])
		histno, _ := strconv.Atoi(args[4])
		cursor, _ := strconv.Atoi(args[5])

		hook := fig_ipc.CreateEditBufferHook(args[0], integrationVersion, args[2], pid, histno, cursor, args[6])
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

var hookInitCmd = &cobra.Command{
	Use:   "init [pid] [tty]",
	Short: "Run the init hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook := fig_ipc.CreateInitHook(pid, args[1])
		fig_ipc.SendHook(hook)
	},
}

var hookKeyboardFocusChanged = &cobra.Command{
	Use:   "keyboard-focus-changed [bundle-id] [focused-session]",
	Short: "Run the keyboard-focus-changed hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		hook := fig_ipc.CreateKeyboardFocusChangedHook(args[0], args[1])
		fig_ipc.SendHook(hook)
	},
}

var hookIntegrationReady = &cobra.Command{
	Use:   "integration-ready [integration]",
	Short: "Run the integration-ready hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateIntegrationReadyHook(args[0])
		fig_ipc.SendHook(hook)
	},
}
