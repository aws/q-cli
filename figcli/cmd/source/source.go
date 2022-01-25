package source

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdSource() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "source",
		Short: "link with terminal session",
		Long:  "(re)connect to this terminal session",
		Run: func(cmd *cobra.Command, arg []string) {
			err := fig_ipc.RestartSettingsListenerCommand()
			if err != nil {
				logging.Log("fig source:", err.Error())
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				os.Exit(1)
			}

			tty, err := diagnostics.GetTty()
			if err != nil {
				logging.Log("fig source:", err.Error())
				fmt.Println("Unable to get TTY")
				os.Exit(1)
			}

			pid := os.Getppid()
			hook, _ := fig_ipc.CreateInitHook(pid, tty)

			fig_ipc.SendHook(hook)

			fmt.Print("\n")
			fmt.Print(lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Bold(true).Render("fig"))
			fmt.Printf(" is now connected to this terminal session. (%s)\n\n", tty)
		},
	}

	return cmd
}
