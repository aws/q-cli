package alpha

import (
	"fig-cli/cmd/alpha/source"
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdAlpha() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "alpha",
		Short: "Open dotfiles",
		Run: func(cmd *cobra.Command, args []string) {
			response, err := fig_ipc.RunOpenUiElementCommand(fig_proto.UiElement_MISSION_CONTROL)
			if err != nil {
				_, err := diagnostics.GetAppInfo()

				if err != nil {
					fmt.Print("\n› Launching Fig...\n\n")
					figExec := exec.Command("open", "-b", "com.mschrage.fig")
					figExec.Run()
					figExec.Process.Release()
				}
				return
			}

			if response != "" {
				fmt.Printf("\n%s\n\n", response)
			}

			fmt.Printf("\n→ Opening dotfiles...\n\n")
		},
	}

	cmd.AddCommand(source.NewCmdSource())

	return cmd
}
