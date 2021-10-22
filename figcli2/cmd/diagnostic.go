package cmd

import (
	"fig-cli/diagnostics"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(diagnosticCmd)
}

var diagnosticCmd = &cobra.Command{
	Use:   "diagnostic",
	Short: "Run diagnostic tests",
	Run: func(cmd *cobra.Command, arg []string) {
		fmt.Println(diagnostics.Summary())
	},
}
