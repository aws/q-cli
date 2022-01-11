package alpha

import (
	"fig-cli/cmd/alpha/source"
	"fig-cli/logging"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdAlpha() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "alpha",
		Short: "Open mission control",
		Run: func(cmd *cobra.Command, args []string) {
			// TODO: send protobuf to open local mission control
			if err := exec.Command("open", "https://fig.io").Run(); err != nil {
				logging.Log("Unable to open mission control", err.Error())
			}
		},
	}

	cmd.AddCommand(source.NewCmdSource())

	return cmd
}
