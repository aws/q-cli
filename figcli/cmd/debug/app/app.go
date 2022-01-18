package app

import (
	"fig-cli/diagnostics"
	"fmt"
	"os/exec"
	"strings"

	"github.com/spf13/cobra"
)

func NewCmdApp() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "app",
		Short: "debug fig app",
		Run: func(cmd *cobra.Command, args []string) {
			_, err := diagnostics.GetAppInfo()

			if err != nil {
				fmt.Println("Fig app is not currently running...")

				execFig := exec.Command("/Applications/Fig.app/Contents/MacOS/fig")
				err := execFig.Start()

				if err != nil {
					fmt.Println("Could not start fig")
				}

				execFig.Process.Release()
				return
			}

			bundelPath, err := exec.Command("lsappinfo", "info", "-only", "bundlepath", "-app", "com.mschrage.fig").Output()
			bundelPathStr := strings.Replace(strings.Split(strings.TrimSpace(string(bundelPath)), "=")[1], "\"", "", -1)

			if err != nil {
				fmt.Println("Could not get Fig app bundle path")
				return
			}

			front, err := exec.Command("lsappinfo", "front").Output()
			frontStr := strings.TrimSpace(string(front))

			if err != nil {
				fmt.Println("Could not get front app")
				return
			}

			terminalEmu, err := exec.Command("lsappinfo", "info", "-only", "name", "-app", frontStr).Output()
			terminalEmuStr := strings.Replace(strings.Split(strings.TrimSpace(string(terminalEmu)), "=")[1], "\"", "", -1)

			if err != nil {
				fmt.Println("Could not get terminal emulator app")
				return
			}

			fmt.Println("Running the Fig.app executable directly from " + bundelPathStr + ".")
			fmt.Println("You will need to grant accessibility permissions to the current terminal (" + terminalEmuStr + ")!")
		},
	}

	return cmd
}
