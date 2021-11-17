package setpath

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/settings"
	"fmt"
	"os"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdSetPath() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "set-path",
		Short: "Set the path to the fig executable",
		Long:  `Set the path to the fig executable`,
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Printf("\n  Setting $PATH variable in Fig pseudo-terminal...\n\n")

			// Get the users $PATH
			path := os.Getenv("PATH")

			// Load ~/.fig/settings.json and set the path
			settings, err := settings.Load()
			if err != nil {
				fmt.Println("Error: ", err)
				return
			}

			settings.Set("pty.path", path)

			// Trigger update of ENV in PTY
			pty, err := diagnostics.GetTty()
			if err != nil {
				fmt.Println("Error: ", err)
				return
			}

			hook, err := fig_ipc.CreateInitHook(os.Getppid(), pty)
			if err != nil {
				fmt.Println("Error: ", err)
				return
			}

			err = fig_ipc.SendHook(hook)
			if err != nil {
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				return
			}
		},
	}

	return cmd
}
