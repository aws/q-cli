package cmd

import (
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(tweetCmd)
}

var tweetCmd = &cobra.Command{
	Use:   "tweet",
	Short: "Tweet about Fig",
	Long:  `Tweet about Fig`,
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},
	Run: func(cmd *cobra.Command, arg []string) {
		fmt.Println("→ Opening Twitter...")
		exec.Command("open", "https://twitter.com/intent/tweet?text=I%27ve%20added%20autocomplete%20to%20my%20terminal%20using%20@fig!%0a%0a%F0%9F%9B%A0%F0%9F%86%95%F0%9F%91%89%EF%B8%8F&url=https://fig.io").Run()
	},
}
