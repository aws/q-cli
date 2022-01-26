package app

import (
	"fig-cli/cmd/app/install"
	"fig-cli/cmd/app/onboarding"
	"fig-cli/cmd/app/running"
	setpath "fig-cli/cmd/app/set-path"
	"fig-cli/cmd/app/uninstall"

	"github.com/spf13/cobra"
)

func NewCmdApp() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "app",
		Short: "Interact with the macOS app",
	}

	cmd.AddCommand(onboarding.NewCmdOnboarding())
	cmd.AddCommand(install.NewCmdInstall())
	cmd.AddCommand(setpath.NewCmdSetPath())
	cmd.AddCommand(running.NewCmdRunning())
	cmd.AddCommand(uninstall.NewCommandUninstall())

	return cmd
}
