package report

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdReport() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "report",
		Short: "Open the report window",
		Run: func(cmd *cobra.Command, arg []string) {
			err := fig_ipc.ReportWindowCommand(strings.Join(arg, " "))
			if err != nil {
				logging.Log("report:", err.Error())
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				os.Exit(1)
			} else {
				fmt.Printf("\n→ Opening report...\n\n")
			}
		},
	}

	return cmd
}
