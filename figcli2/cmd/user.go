package cmd

import (
	"fmt"
	"os/exec"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func init() {
	userCmd.AddCommand(userLoginCmd)
	userCmd.AddCommand(userLogoutCmd)
	userCmd.AddCommand(userWhoamiCmd)

	rootCmd.AddCommand(userCmd)
}

var userCmd = &cobra.Command{
	Use:   "user",
	Short: "update repo of completion scripts",
}

var userLogoutCmd = &cobra.Command{
	Use:   "logout",
	Short: "logout from fig",
	Run: func(cmd *cobra.Command, arg []string) {
		// TODO
	},
}

var userLoginCmd = &cobra.Command{
	Use:   "login",
	Short: "login to fig",
	Run: func(cmd *cobra.Command, arg []string) {
		// TODO
	},
}

var userWhoamiCmd = &cobra.Command{
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
