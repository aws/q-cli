package cmd

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"strconv"
	"strings"

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
	rootCmd.AddCommand(vscodeLegacy)

	rootCmd.AddCommand(testCmd)

	rootCmd.AddCommand(legacyAppRunning)
	rootCmd.AddCommand(legacyUpdate)
}

var legacyZshKeybuffer = &cobra.Command{
	Use:                "bg:zsh-keybuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
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
			logging.Log(err.Error())
			return
		}

		err = fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}

var legacyFishKeybuffer = &cobra.Command{
	Use:                "bg:fish-keybuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
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
			logging.Log(err.Error())
			return
		}

		err = fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}

var legacyBashKeybuffer = &cobra.Command{
	Use:                "bg:bash-keybuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
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
			logging.Log(err.Error())
			return
		}

		err = fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}

var legacyPrompt = &cobra.Command{
	Use:                "bg:prompt [pid] [tty]",
	Short:              "Run the prompt hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook, err := fig_ipc.CreatePromptHook(pid, args[1])
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

var legacyInit = &cobra.Command{
	Use:                "bg:init [pid] [tty]",
	Short:              "Run the init hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 2 {
			return
		}

		pid, _ := strconv.Atoi(args[0])

		hook, err := fig_ipc.CreateInitHook(pid, args[1])
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

var legacyItermReady = &cobra.Command{
	Use:                "bg:iterm-api-ready",
	Short:              "Run the integration-ready hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		hook := fig_ipc.CreateIntegrationReadyHook("iterm")
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}

var legacyHide = &cobra.Command{
	Use:                "bg:hide",
	Short:              "Run the hide hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		hook := fig_ipc.CreateHideHook()
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}

var legacyEvent = &cobra.Command{
	Use:                "bg:event [event-name]",
	Short:              "Run the event hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		hook := fig_ipc.CreateEventHook(args[0])
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}

var legacyClearKeybuffer = &cobra.Command{
	Use:                "bg:clear-keybuffer",
	Short:              "Run the clear-keybuffer hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
	},
}

var legacyHyper = &cobra.Command{
	Use:                "bg:hyper [focused-session-id]",
	Short:              "Run the keyboard-focus-changed hook",
	DisableFlagParsing: true,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		id := strings.TrimLeft(args[0], "hyper:")

		hook := fig_ipc.CreateKeyboardFocusChangedHook("hyper", id)
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
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
	Use:                "bg:exec [pid] [tty]",
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

var vscodeLegacy = &cobra.Command{
	Use:   "bg:vscode [focused-session-id]",
	Short: "Run the vscode hook",
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) != 1 {
			return
		}

		tabId := strings.TrimPrefix(args[0], "code:")

		hook := fig_ipc.CreateKeyboardFocusChangedHook("code", tabId)
		err := fig_ipc.SendHook(hook)
		if err != nil {
			logging.Log(err.Error())
		}
	},
}
