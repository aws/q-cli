package report

import (
	fig_ipc "fig-cli/fig-ipc"
	"strings"

	"github.com/spf13/cobra"
)

func NewCmdReport() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "report",
		Short: "Open the report window",
		Run: func(cmd *cobra.Command, arg []string) {
			fig_ipc.ReportWindowCommand(strings.Join(arg, " "))
		},
	}

	return cmd
}
