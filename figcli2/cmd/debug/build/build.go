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
		Use:   "build",
		Short: "Switch Build",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, arg []string) {
			err := fig_ipc.RunBuildCommand(arg[0])
			if err != nil {
				logging.Log("fig debug build:", err.Error())
				fmt.Printf("\n" + lipgloss.NewStyle().Bold(true).Render("Unable to Switch Build") +
					"\n\n" + "Fig might not be running, you can run " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("#ff00ff")).Render("fig") +
					" to launch it\n\n")
				os.Exit(1)
			}
		},
	}

	return cmd
}
