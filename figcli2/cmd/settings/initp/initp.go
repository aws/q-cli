package initp

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdInit() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "init",
		Short: "Reload the settings listener",
		Run: func(cmd *cobra.Command, arg []string) {
			err := fig_ipc.RestartSettingsListenerCommand()
			if err != nil {
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				return
			}

			fmt.Printf("\nSettings listener restarted.\n\n")
		},
	}

	return cmd
}
