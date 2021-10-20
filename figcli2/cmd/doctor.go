package cmd

import (
	"fmt"
	"os"
	"os/user"
	"strings"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(doctorCmd)
}

var doctorCmd = &cobra.Command{
	Use:   "doctor",
	Short: "Check Fig is properly configured",
	Long:  "Runs a series of checks to ensure Fig is properly configured",
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println()
		fmt.Println("Let's make sure Fig is running...")
		fmt.Println()

		// Get user
		user, err := user.Current()
		if err != nil {
			fmt.Println(err)
			return
		}

		// Check if file ~/.fig/bin/fig exists
		if _, err := os.ReadFile(fmt.Sprintf("%s/.fig/bin/fig", user.HomeDir)); err != nil {
			fmt.Println("Fig bin exists:\t❌")
		} else {
			fmt.Println("Fig bin exists:\t✅")
		}

		// Check if fig is in PATH
		path := os.Getenv("PATH")
		if !strings.Contains(path, "/.fig/bin") {
			fmt.Println("Fig in PATH:\t❌")
		} else {
			fmt.Println("Fig in PATH:\t✅")
		}

		for _, file := range []string{".profile", ".zprofile", ".bash_profile", ".bashrc", ".zshrc"} {
			// Read file if it exists
			if _, err := os.ReadFile(fmt.Sprintf("%s/.fig/%s", user.HomeDir, file)); err != nil {
				fmt.Printf("%s exists:\t❌\n", file)
			} else {
				fmt.Printf("%s exists:\t✅\n", file)
			}
		}
	},
}
