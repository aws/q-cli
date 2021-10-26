package cmd

import (
	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(bgInitCmd)
}

var bgInitCmd = &cobra.Command{
	Use: "bg:init",
	Run: func(cmd *cobra.Command, arg []string) {
	},
}
