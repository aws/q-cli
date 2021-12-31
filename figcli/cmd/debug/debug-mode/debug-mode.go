package debugmode

import (
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdDebugMode() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "debug-mode [mode]",
		Short: "Toggle/set debug mode",
		Long:  "Toggle/set debug mode",
		ValidArgs: []string{
			"on",
			"off",
		},
		Args: cobra.MaximumNArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				resp, err := fig_ipc.ToggleDebugModeCommand()
				if err != nil {
					logging.Log("Debug mode toggle error:", err.Error())
					fmt.Println("Could not toggle debug mode")
					return
				}

				fmt.Println("Debug mode:", resp)
				return
			}

			mode := args[0]
			if mode == "on" {
				_, err := fig_ipc.SetDebugModeCommand(true)
				if err != nil {
					logging.Log("Debug mode set error:", err.Error())
					fmt.Println("Could not set debug mode")
					return
				}
			} else if mode == "off" {
				_, err := fig_ipc.SetDebugModeCommand(false)
				if err != nil {
					logging.Log("Debug mode set error:", err.Error())
					fmt.Println("Could not set debug mode")
					return
				}
			} else {
				fmt.Println("Unknown mode:", mode)
				fmt.Println("Valid modes: on, off")
			}
		},
	}

	return cmd
}
