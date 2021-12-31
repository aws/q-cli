package verify

import (
	"fig-cli/cmd/integrations"
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdVerify() *cobra.Command {
	cmd := &cobra.Command{
		Use:       "verify [integration]",
		Short:     "Verify Fig integrations",
		Long:      "Verify Fig integrations",
		Args:      cobra.ExactValidArgs(1),
		ValidArgs: integrations.IntegrationList,
		Run: func(cmd *cobra.Command, args []string) {
			integration := args[0]
			if res, err := fig_ipc.IntegrationVerifyInstall(integrations.IntegrationMap[integration]); err != nil {
				fmt.Println("Error verifying integration:", err.Error())
			} else {
				fmt.Println("Status:", res)
			}
		},
	}

	return cmd
}
