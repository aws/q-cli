package cmd

import (
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(issueCmd)
}

var issueCmd = &cobra.Command{
	Use:   "issue",
	Short: "Create a new GitHub issue",
	Long:  "Create a new GitHub issue in withfig/fig.",
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},

	Run: func(cmd *cobra.Command, arg []string) {
		fmt.Println("→ Opening GitHub...")
		exec.Command("open", "https://github.com/withfig/fig/issues/new").Run()
	},
}
