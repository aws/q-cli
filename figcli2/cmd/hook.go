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
	hookCmd.AddCommand(hookKeyboardFocusChangedCmd)
	hookCmd.AddCommand(hookIntegrationReadyCmd)
	hookCmd.AddCommand(hookHideCmd)
	hookCmd.AddCommand(hookEventCmd)

	rootCmd.AddCommand(hookCmd)
}

// TODO: Add error handling for hooks

var hookCmd = &cobra.Command{
	Use:    "hook",
	Short:  "Run a hook",
	Hidden: true,
}

var hookEditbufferCmd = &cobra.Command{
	Use:   "editbuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
	Short: "Run the editbuffer hook",
	DisableFlagParsing: true,
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
	},
}

var hookPromptCmd = &cobra.Command{
	Use:   "prompt [pid] [tty]",
	Short: "Run the prompt hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook := fig_ipc.CreatePromptHook(pid, args[1])
		fig_ipc.SendHook(hook)
	},
}

var hookInitCmd = &cobra.Command{
	Use:   "init [pid] [tty]",
	Short: "Run the init hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook := fig_ipc.CreateInitHook(pid, args[1])
		fig_ipc.SendHook(hook)
	},
}

var hookKeyboardFocusChangedCmd = &cobra.Command{
	Use:   "keyboard-focus-changed [bundle-id] [focused-session-id]",
	Short: "Run the keyboard-focus-changed hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		hook := fig_ipc.CreateKeyboardFocusChangedHook(args[0], args[1])
		fig_ipc.SendHook(hook)
	},
}

var hookIntegrationReadyCmd = &cobra.Command{
	Use:   "integration-ready [integration]",
	Short: "Run the integration-ready hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateIntegrationReadyHook(args[0])
		fig_ipc.SendHook(hook)
	},
}

var hookHideCmd = &cobra.Command{
	Use:   "hide",
	Short: "Run the hide hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		hook := fig_ipc.CreateHideHook()
		fig_ipc.SendHook(hook)
	},
}

var hookEventCmd = &cobra.Command{
	Use:   "event [event-name]",
	Short: "Run the event hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateEventHook(args[0])
		fig_ipc.SendHook(hook)
	},
}
