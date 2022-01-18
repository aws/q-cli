package install

import (
	"fig-cli/cmd/integrations"
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdInstall() *cobra.Command {
	cmd := &cobra.Command{
		Use:       "install [integration]",
		Short:     "Install Fig integrations",
		Long:      "Install Fig integrations",
		Args:      cobra.ExactValidArgs(1),
		ValidArgs: integrations.IntegrationList,
		Run: func(cmd *cobra.Command, args []string) {
			integration := args[0]
			if res, err := fig_ipc.IntegrationInstall(integrations.IntegrationMap[integration]); err != nil {
				fmt.Println("Error installing integration:", err.Error())
			} else {
				fmt.Println(res)
			}
		},
	}

	return cmd
}
