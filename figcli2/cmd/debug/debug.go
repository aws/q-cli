package debug

import (
	"fig-cli/cmd/debug/app"
	"fig-cli/cmd/debug/build"
	debugmode "fig-cli/cmd/debug/debug-mode"
	"fig-cli/cmd/debug/diagnostic"
	"fig-cli/cmd/debug/dotfiles"
	"fig-cli/cmd/debug/logs"
	"fig-cli/cmd/debug/prefs"
	promptaccessibility "fig-cli/cmd/debug/prompt-accessibility"
	"fig-cli/cmd/debug/sample"
	"fig-cli/cmd/debug/ssh"
	"fig-cli/cmd/debug/terminal"
	unixsocket "fig-cli/cmd/debug/unix-socket"
	verifycodesign "fig-cli/cmd/debug/verify-codesign"

	"github.com/spf13/cobra"
)

func NewCmdDebug() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "debug",
		Short: "debug",
	}

	cmd.AddCommand(app.NewCmdApp())
	cmd.AddCommand(build.NewCmdBuild())
	cmd.AddCommand(debugmode.NewCmdDebugMode())
	cmd.AddCommand(diagnostic.NewCmdDiagnostic())
	cmd.AddCommand(dotfiles.NewCmdDotfiles())
	cmd.AddCommand(logs.NewCmdLogs())
	cmd.AddCommand(prefs.NewCmdPrefs())
	cmd.AddCommand(promptaccessibility.NewCmdPromptAccessibility())
	cmd.AddCommand(sample.NewCmdSample())
	cmd.AddCommand(ssh.NewCmdSsh())
	cmd.AddCommand(terminal.NewCmdTerminal())
	cmd.AddCommand(unixsocket.NewCmdUnixSocket())
	cmd.AddCommand(verifycodesign.NewCmdVerifyCodesign())

	return cmd
}
