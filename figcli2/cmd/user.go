package cmd

import (
	"fmt"
	"os/exec"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func init() {
	userCmd.AddCommand(logoutCmd)
	userCmd.AddCommand(getCmd)

	rootCmd.AddCommand(userCmd)
}

var userCmd = &cobra.Command{
	Use:   "user",
	Short: "update repo of completion scripts",
}

var logoutCmd = &cobra.Command{
	Use:   "logout",
	Short: "logout from fig",
	Run: func(cmd *cobra.Command, arg []string) {

	},
}

var getCmd = &cobra.Command{
	Use:   "get",
	Short: "get user info",
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
