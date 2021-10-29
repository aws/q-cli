package cmd

import (
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(communityCmd)
}

var communityCmd = &cobra.Command{
	Use:   "community",
	Short: "Join the Fig community",
	Run: func(cmd *cobra.Command, arg []string) {
		fmt.Println("→ Joining Fig community....")
		exec.Command("open", "https://fig.io/community").Run()
	},
}
