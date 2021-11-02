package app

import (
	installandupgrade "fig-cli/cmd/app/install-and-upgrade"
	"fig-cli/cmd/app/onboarding"
	"fig-cli/cmd/app/running"
	setpath "fig-cli/cmd/app/set-path"
	"fig-cli/cmd/app/theme"
	"fig-cli/cmd/app/uninstall"

	"github.com/spf13/cobra"
)

func NewCmdApp() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "app",
		Short: "Manage your Fig app",
		Annotations: map[string]string{
			"figcli.command.categories": "Common",
		},
	}

	cmd.AddCommand(onboarding.NewCmdOnboarding())
	cmd.AddCommand(theme.NewThemeCmd())
	cmd.AddCommand(installandupgrade.NewCmdInstallAndUpgrade())
	cmd.AddCommand(setpath.NewCmdSetPath())
	cmd.AddCommand(running.NewCommandRunning())
	cmd.AddCommand(uninstall.NewCommandUninstall())

	return cmd
}
