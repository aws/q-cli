package cmd

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fmt"
	"strconv"

	"github.com/spf13/cobra"
)

// ============================================================
//
//  LEGACY HOOKS: TO BE REMOVED IN THE NEXT PRODUCTION RELEASE
//
// ============================================================

func init() {
	legacyUpdate.Flags().BoolP("force", "f", false, "Force update")

	rootCmd.AddCommand(legacyInit)
	rootCmd.AddCommand(legacyItermReady)
	rootCmd.AddCommand(legacyZshKeybuffer)
	rootCmd.AddCommand(legacyFishKeybuffer)
	rootCmd.AddCommand(legacyBashKeybuffer)
	rootCmd.AddCommand(legacyPrompt)
	rootCmd.AddCommand(legacyHide)
	rootCmd.AddCommand(legacyEvent)
	rootCmd.AddCommand(legacyClearKeybuffer)
	rootCmd.AddCommand(legacyHyper)
	rootCmd.AddCommand(legacyExec)

	rootCmd.AddCommand(testCmd)

	rootCmd.AddCommand(legacyAppRunning)
	rootCmd.AddCommand(legacyUpdate)
}

var legacyZshKeybuffer = &cobra.Command{
	Use:   "bg:zsh-keybuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
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
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyFishKeybuffer = &cobra.Command{
	Use:   "bg:fish-keybuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
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
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyBashKeybuffer = &cobra.Command{
	Use:   "bg:bash-keybuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
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
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyPrompt = &cobra.Command{
	Use:   "bg:prompt [pid] [tty]",
	Short: "Run the prompt hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook := fig_ipc.CreatePromptHook(pid, args[1])
		_ = fig_ipc.SendHook(hook)
		// TODO: Add error handling
	},
}

var legacyInit = &cobra.Command{
	Use:   "bg:init [pid] [tty]",
	Short: "Run the init hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook := fig_ipc.CreateInitHook(pid, args[1])
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyItermReady = &cobra.Command{
	Use:   "bg:iterm-api-ready",
	Short: "Run the integration-ready hook",
	Run: func(cmd *cobra.Command, args []string) {
		hook := fig_ipc.CreateIntegrationReadyHook("iterm")
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyHide = &cobra.Command{
	Use:   "bg:hide",
	Short: "Run the hide hook",
	Run: func(cmd *cobra.Command, args []string) {
		hook := fig_ipc.CreateHideHook()
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyEvent = &cobra.Command{
	Use:   "bg:event [event-name]",
	Short: "Run the event hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateEventHook(args[0])
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyClearKeybuffer = &cobra.Command{
	Use:   "bg:clear-keybuffer",
	Short: "Run the clear-keybuffer hook",
	Run: func(cmd *cobra.Command, args []string) {
	},
}

var legacyHyper = &cobra.Command{
	Use:   "keyboard-focus-changed [focused-session-id]",
	Short: "Run the keyboard-focus-changed hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateKeyboardFocusChangedHook("co.zeit.hyper", args[1])
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyAppRunning = &cobra.Command{
	Use:   "app:running",
	Short: "Check if the app is running",
	Run: func(cmd *cobra.Command, args []string) {
		appInfo, err := diagnostics.GetAppInfo()
		if err != nil {
			return
		}

		if appInfo.IsRunning() {
			fmt.Println("1")
		} else {
			fmt.Println("0")
		}
	},
}

var legacyExec = &cobra.Command{
	Use:   "bg:exec [pid] [tty]",
	Short: "Run the exec hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook := fig_ipc.CreatePreExecHook(pid, args[1])
		_ = fig_ipc.SendHook(hook)
	},
}

var legacyUpdate = &cobra.Command{
	Use:   "update:app [pid] [tty]",
	Short: "Run the update command",
	Run: func(cmd *cobra.Command, args []string) {
		fig_ipc.UpdateCommand(true)
	},
}

var testCmd = &cobra.Command{
	Use:   "test",
	Short: "Run a test",
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("test")
	},
}
