package launch

import (
	"fig-cli/diagnostics"
	"fig-cli/logging"
	"fmt"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func Launch() {
	if diagnostics.IsFigRunning() {
		fmt.Printf("\n→ Fig is already running.\n\n")
		return
	}

	fmt.Printf("\n→ Launching Fig...\n\n")
	figCmd := exec.Command("open", "-g", "-b", "com.mschrage.fig")

	if err := figCmd.Run(); err != nil {
		fmt.Printf("\n→ Fig could not be launched.\n\n")
		logging.Log("fig launch:", err.Error())
		os.Exit(1)
	}

	if err := figCmd.Process.Release(); err != nil {
		fmt.Printf("\n→ Fig could not be launched.\n\n")
		logging.Log("fig launch:", err.Error())
		os.Exit(1)
	}
}

func NewCmdLaunch() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "launch",
		Short:  "Launch Fig",
		Run: func(cmd *cobra.Command, args []string) {
			Launch()
		},
	}

	return cmd
}
