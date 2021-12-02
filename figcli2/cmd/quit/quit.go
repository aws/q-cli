package quit

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/logging"
	"fmt"
	"os"
	"os/exec"
	"strconv"
	"time"

	"github.com/spf13/cobra"
)

func NewCmdQuit() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "quit",
		Short: "Quit Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			appInfo, appInfoErr := diagnostics.GetAppInfo()

			if !appInfo.IsRunning() {
				fmt.Printf("\n→ Fig is not running\n\n")
				return
			}

			fmt.Printf("\n→ Quitting Fig...\n\n")
			if err := fig_ipc.QuitCommand(); err != nil {
				logging.Log("fig quit:", err.Error())

				// Try again after a short delay
				time.Sleep(500 * time.Millisecond)

				if err := fig_ipc.QuitCommand(); err != nil {
					// Kill the process if it's still running
					if appInfoErr != nil {
						logging.Log("fig quit:", err.Error())
						fmt.Println(err.Error())
						fmt.Printf("\nUnable to quit Fig\n\n")
						os.Exit(1)
					}

					pid, err := appInfo.Pid()
					if err != nil {
						logging.Log("fig quit:", err.Error())
						fmt.Printf("\nUnable to quit Fig\n\n")
						os.Exit(1)
					}

					quitCmd := exec.Command("kill", "-KILL", strconv.Itoa(pid))
					err = quitCmd.Run()
					if err != nil {
						logging.Log("fig quit:", err.Error())
						fmt.Printf("\nUnable to quit Fig\n\n")
						os.Exit(1)
					}
				}
			}
		},
	}

	return cmd
}
