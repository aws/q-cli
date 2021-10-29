package cmd

import (
	"fig-cli/diagnostics"
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(launchCmd)
}

var launchCmd = &cobra.Command{
	Use:   "launch",
	Short: "Launch Fig",
	Long:  "Launch Fig",
	Run: func(cmd *cobra.Command, arg []string) {
		_, err := diagnostics.GetAppInfo()

		if err != nil {
			fmt.Print("\n› Launching Fig...\n\n")
			figExec := exec.Command("open", "-b", "com.mschrage.fig")
			figExec.Run()
			figExec.Process.Release()
		} else {
			fmt.Print("\n› It seems like the Fig is already running.\n\n")
		}

	},
}
