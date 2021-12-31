package inith

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"strconv"

	"github.com/spf13/cobra"
)

func NewCmdInit() *cobra.Command {
	cmd := &cobra.Command{
		Use:                "init [pid] [tty]",
		Short:              "Run the init hook",
		DisableFlagParsing: true,
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) != 2 {
				return
			}

			pid, _ := strconv.Atoi(args[0])

			hook, err := fig_ipc.CreateInitHook(pid, args[1])
			if err != nil {
				logging.Log("Error:", err.Error())
				return
			}

			err = fig_ipc.SendHook(hook)
			if err != nil {
				logging.Log("Error:", err.Error())
				return
			}
		},
	}

	return cmd
}
