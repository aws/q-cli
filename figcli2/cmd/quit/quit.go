package quit

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdQuit() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "quit",
		Short: "Quit Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			if err := fig_ipc.QuitCommand(); err != nil {
				logging.Log("restart:", err.Error())
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				os.Exit(1)
			} else {
				fmt.Printf("\n→ Quitting Fig...\n\n")
			}
		},
	}

	return cmd
}
