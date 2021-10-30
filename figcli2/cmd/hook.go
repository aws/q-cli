package cmd

import (
	"strconv"

	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"

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
	hookCmd.AddCommand(hookPreExecCmd)

	rootCmd.AddCommand(hookCmd)
}

var hookCmd = &cobra.Command{
	Use:    "hook",
	Short:  "Run a hook",
	Hidden: true,
}

var hookEditbufferCmd = &cobra.Command{
	Use:                "editbuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
	Short:              "Run the editbuffer hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 7 {
			return
		}

		integrationVersion, _ := strconv.Atoi(args[1])
		pid, _ := strconv.Atoi(args[3])
		histno, _ := strconv.Atoi(args[4])
		cursor, _ := strconv.Atoi(args[5])

		hook, err := fig_ipc.CreateEditBufferHook(args[0], integrationVersion, args[2], pid, histno, cursor, args[6])
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}

		err = fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log("Error:", err.Error())
		}
	},
}

var hookPromptCmd = &cobra.Command{
	Use:                "prompt [pid] [tty]",
	Short:              "Run the prompt hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook, err := fig_ipc.CreatePromptHook(pid, args[1])
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}

		err = fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}
	},
}

var hookInitCmd = &cobra.Command{
	Use:                "init [pid] [tty]",
	Short:              "Run the init hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook, err := fig_ipc.CreateInitHook(pid, args[1])
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}

		err = fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}
	},
}

var hookKeyboardFocusChangedCmd = &cobra.Command{
	Use:                "keyboard-focus-changed [bundle-id] [focused-session-id]",
	Short:              "Run the keyboard-focus-changed hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		hook := fig_ipc.CreateKeyboardFocusChangedHook(args[0], args[1])
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}
	},
}

var hookIntegrationReadyCmd = &cobra.Command{
	Use:                "integration-ready [integration]",
	Short:              "Run the integration-ready hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateIntegrationReadyHook(args[0])
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}
	},
}

var hookHideCmd = &cobra.Command{
	Use:                "hide",
	Short:              "Run the hide hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		hook := fig_ipc.CreateHideHook()
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}
	},
}

var hookEventCmd = &cobra.Command{
	Use:                "event [event-name]",
	Short:              "Run the event hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateEventHook(args[0])
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log("Error:", err.Error())
			return
		}
	},
}

var hookPreExecCmd = &cobra.Command{
	Use:                "pre-exec [pid] [tty]",
	Short:              "Run the exec hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook, err := fig_ipc.CreatePreExecHook(pid, args[1])
		if err != nil {
			logging.Log(err.Error())
			return
		}

		err = fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}
