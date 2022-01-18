package ime

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

func NewCmdInputMethod() *cobra.Command {
	cmd := &cobra.Command{
		Use:       "ime",
		Short:     "Input Method",
		Long:      "Perform commands on input method editor",
		ValidArgs: []string{"install", "uninstall", "select", "deselect", "enable", "disable", "status", "register"},
		Args:      cobra.ExactValidArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			res, err := fig_ipc.InputMethodCommand(args[0])
			if err != nil {
				logging.Log("fig debug ime", err.Error())
				fmt.Println("Could not run ime command")
				os.Exit(1)
			}

			fmt.Println(res)
		},
	}

	return cmd
}
