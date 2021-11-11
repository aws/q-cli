package editbuffer

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"strconv"

	"github.com/spf13/cobra"
)

func NewCmdEditbuffer() *cobra.Command {
	cmd := &cobra.Command{
		Use:                "editbuffer [session-id] [integration] [tty] [pid] [histno] [cursor] [text]",
		Short:              "Run the editbuffer hook",
		DisableFlagParsing: true,
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) != 7 {
				return
			}

			integrationVersion, _ := strconv.Atoi(args[1])
			pid, _ := strconv.Atoi(args[3])
			histno, _ := strconv.Atoi(args[4])
			cursor, _ := strconv.Atoi(args[5])

			hook, err := fig_ipc.CreateEditBufferHook(args[0], integrationVersion, args[2], pid, histno, cursor, args[6])
			if err != nil {
				logging.Log("Error:", err.Error())
				return
			}

			err = fig_ipc.SendHook(hook)
			if err != nil {
				logging.Log("Error:", err.Error())
			}
		},
	}

	return cmd
}
