package build

import (
	fig_ipc "fig-cli/fig-ipc"

	"github.com/spf13/cobra"
)

func NewCmdBuild() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "build",
		Short: "Switch branch",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, arg []string) {
			err := fig_ipc.RunBuildCommand(arg[0])
			if err != nil {
				panic(err)
			}
		},
	}

	return cmd
}
