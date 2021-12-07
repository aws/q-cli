package setpath

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
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
			fmt.Printf("\nSetting $PATH variable in Fig pseudo-terminal...\n\n")

			// Get the users $PATH
			path := os.Getenv("PATH")

			// Load ~/.fig/settings.json and set the path
			settings, err := settings.Load()
			if err != nil {
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("1")).Render("Error:") + " Unable to load settings file")
				logging.Log("fig app set-path", err.Error())
				return
			}

			settings.Set("pty.path", path)

			if err := settings.Save(); err != nil {
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("1")).Render("Error:"), "Unable to save settings file")
				logging.Log("fig app set-path", err.Error())
				return
			}

			fmt.Printf("Fig will now use the following path to locate the fig executable:\n" +
				lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render(path) +
				"\n\n")

			// Trigger update of ENV in PTY
			pty, err := diagnostics.GetTty()
			if err != nil {
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("1")).Render("Error:") + " Could not reload, to use new path restart your terminal")
				logging.Log("fig app set-path", err.Error())
				return
			}

			hook, err := fig_ipc.CreateInitHook(os.Getppid(), pty)
			if err != nil {
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("1")).Render("Error:") + " Could not reload, to use new path restart your terminal")
				logging.Log("fig app set-path", err.Error())
				return
			}

			err = fig_ipc.SendHook(hook)
			if err != nil {
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				logging.Log("fig app set-path", err.Error())
				return
			}
		},
	}

	return cmd
}
