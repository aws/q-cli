package uninstall

import (
	"fig-cli/cmd/integrations"
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdUninstall() *cobra.Command {
	cmd := &cobra.Command{
		Use:       "uninstall [integration]",
		Short:     "Uninstall Fig integrations",
		Long:      "Uninstall Fig integrations",
		Args:      cobra.ExactValidArgs(1),
		ValidArgs: integrations.IntegrationList,
		Run: func(cmd *cobra.Command, args []string) {
			integration := args[0]
			if res, err := fig_ipc.IntegrationUninstall(integrations.IntegrationMap[integration]); err != nil {
				fmt.Println("Error uninstalling integration:", err.Error())
			} else {
				fmt.Println(res)
			}
		},
	}

	return cmd
}
