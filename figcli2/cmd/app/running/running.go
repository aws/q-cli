package running

import (
	"fig-cli/diagnostics"
	"fmt"

	"github.com/spf13/cobra"
)

func running() string {
	appInfo, err := diagnostics.GetAppInfo()
	if err != nil {
		return "unknown"
	}

	if appInfo.IsRunning() {
		return "1"
	} else {
		return "0"
	}
}

func NewCmdRunning() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "running",
		Short: "Gets the status if Fig is running",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println(running())
		},
	}

	return cmd
}
