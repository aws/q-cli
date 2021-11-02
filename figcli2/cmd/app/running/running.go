package running

import (
	"fig-cli/diagnostics"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCommandRunning() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "running",
		Short: "Gets the status if Fig is running",
		Run: func(cmd *cobra.Command, args []string) {
			appInfo, err := diagnostics.GetAppInfo()
			if err != nil {
				return
			}

			if appInfo.IsRunning() {
				fmt.Println("1")
			} else {
				fmt.Println("0")
			}
		},
	}

	return cmd
}
