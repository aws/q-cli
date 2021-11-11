package launch

import (
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdLaunch() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "launch",
		Short:  "Launch Fig",
		Hidden: true,
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("→ Launching Fig...")
			figCmd := exec.Command("open", "-b", "com.mschrage.fig")
			figCmd.Run()
			figCmd.Process.Release()
		},
	}

	return cmd
}
