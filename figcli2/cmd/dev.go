package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func init() {
	devCmd.AddCommand(docsCmd)
	devCmd.AddCommand(devBuildCmd)

	rootCmd.AddCommand(devCmd)
}

var devCmd = &cobra.Command{
	Use:   "dev",
	Short: "dev",
}

var docsCmd = &cobra.Command{
	Use:   "docs",
	Short: "documentation for building completion specs",
	Run: func(cmd *cobra.Command, arg []string) {
		fmt.Println("→ Opening docs in browser...")
		exec.Command("open", "https://fig.io/docs/getting-started").Run()
	},
}

var devBuildCmd = &cobra.Command{
	Use:   "build",
	Short: "Switch branch",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, arg []string) {
		err := fig_ipc.RunBuildCommand(arg[0])
		if err != nil {
			panic(err)
		}
	},
}
