package cmd

import (
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(contributeCmd)
}

var contributeCmd = &cobra.Command{
	Use:   "contibute",
	Short: "Contribute to Fig Autocomplete",
	Run: func(cmd *cobra.Command, arg []string) {
		fmt.Println("→ Opening GitHub repo...")
		exec.Command("open", "https://github.com/withfig/autocomplete").Run()
	},
}
