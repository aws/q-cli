package logout

import (
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdLogout() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "logout",
		Short: "Logout of Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			// Logout
			logout := fig_proto.Command{
				Command: &fig_proto.Command_Logout{},
			}

			res, err := fig_ipc.SendRecvCommand(&logout)
			if err != nil {
				logging.Log("logout", err.Error())
				fmt.Printf("\n" +
					lipgloss.NewStyle().Bold(true).Render("Unable to Connect to Fig") +
					"\nFig might not be running, to launch Fig run: " +
					lipgloss.NewStyle().Foreground(lipgloss.Color("#ff00ff")).Render("fig launch") +
					"\n\n")
				os.Exit(1)
			}

			message, err := fig_ipc.GetCommandResponseMessage(res)
			if err != nil {
				fmt.Println("Error:", err)
				return
			}

			fmt.Println(message)

			// Restart Fig
			if err := fig_ipc.RestartCommand(); err != nil {
				fmt.Println("Error:", err)
				return
			}
		},
	}

	return cmd
}
