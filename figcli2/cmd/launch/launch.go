package launch

import (
	"fig-cli/logging"
	"fmt"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func Launch() {
	fmt.Printf("\n→ Launching Fig...\n\n")
	figCmd := exec.Command("open", "-b", "com.mschrage.fig")

	if err := figCmd.Run(); err != nil {
		fmt.Printf("\n→ Fig could not be launched.\n\n")
		logging.Log("restart:", err.Error())
		os.Exit(1)
	}

	if err := figCmd.Process.Release(); err != nil {
		fmt.Printf("\n→ Fig could not be launched.\n\n")
		logging.Log("restart:", err.Error())
		os.Exit(1)
	}
}

func NewCmdLaunch() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "launch",
		Short:  "Launch Fig",
		Hidden: true,
		Run: func(cmd *cobra.Command, args []string) {
			Launch()
		},
	}

	return cmd
}
