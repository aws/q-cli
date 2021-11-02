package whoami

import (
	"fmt"
	"os/exec"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdWhoami() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "whoami",
		Short: "get currently logged in user",
		Run: func(cmd *cobra.Command, arg []string) {
			email, err := exec.Command("defaults", "read", "com.mschrage.fig", "userEmail").Output()
			emailStr := strings.TrimSpace(string(email))

			if err != nil || emailStr == "" {
				fmt.Println("No user logged in")
				return
			}

			fmt.Println(lipgloss.NewStyle().Bold(true).Render("Logged in as: ") + emailStr)
		},
	}

	return cmd
}
