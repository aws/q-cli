package build

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdBuild() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "build {dev|prod|staging}",
		Short: "Switch build",
		Long:  `Switch build to dev, staging or prod`,
		ValidArgs: []string{
			"dev",
			"prod",
			"staging",
		},
		Args: cobra.ExactValidArgs(1),
		Run: func(cmd *cobra.Command, arg []string) {
			err := fig_ipc.RunBuildCommand(arg[0])
			if err != nil {
				logging.Log("fig debug build:", err.Error())
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				os.Exit(1)
			}
		},
	}

	return cmd
}
