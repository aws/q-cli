package update

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdUpdate() *cobra.Command {
	var force bool

	cmd := &cobra.Command{
		Use:   "update",
		Short: "Update Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			if err := fig_ipc.UpdateCommand(force); err != nil {
				logging.Log("fig update:", err.Error())
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig launch") +
					"\n\n")
				os.Exit(1)
			} else {
				fmt.Printf("\n→ Checking for updates to macOS app...\n\n")
			}
		},
	}

	cmd.Flags().BoolVarP(&force, "force", "f", false, "Force update")

	return cmd
}
