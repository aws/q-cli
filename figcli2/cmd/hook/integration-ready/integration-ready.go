package integrationready

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"

	"github.com/spf13/cobra"
)

func NewCmdIntegrationReady() *cobra.Command {
	cmd := &cobra.Command{
		Use:                "integration-ready [integration]",
		Short:              "Run the integration-ready hook",
		DisableFlagParsing: true,
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) != 1 {
				return
			}

			hook := fig_ipc.CreateIntegrationReadyHook(args[0])
			err := fig_ipc.SendHook(hook)
			if err != nil {
				logging.Log("Error:", err.Error())
				return
			}
		},
	}

	return cmd
}
