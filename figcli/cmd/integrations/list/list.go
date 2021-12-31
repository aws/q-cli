package list

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdList() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "list",
		Short: "List Fig integrations",
		Long:  "List Fig integrations",
		Run: func(cmd *cobra.Command, args []string) {
			res, err := fig_ipc.GetIntegrations()
			if err != nil {
				fmt.Println("Error:", err)
				return
			}

			for _, integration := range res {
				fmt.Printf("%s: %s\n", integration.GetName(), integration.GetStatus())
			}
		},
	}

	return cmd
}
