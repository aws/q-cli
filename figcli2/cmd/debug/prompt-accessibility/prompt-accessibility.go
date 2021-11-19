package promptaccessibility

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

func NewCmdPromptAccessibility() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "prompt-accessibility",
		Short: "Prompt accessibility",
		Long:  "Prompt for accessibility permissions",
		Run: func(cmd *cobra.Command, args []string) {
			if err := fig_ipc.PromptAccessibilityCommand(); err != nil {
				logging.Log("fig debug prompt-accessibility", err.Error())
				fmt.Println("Could not prompt for accessibility permissions")
				os.Exit(1)
			}
		},
	}

	return cmd
}
