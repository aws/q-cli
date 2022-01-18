package prefs

import (
	"fmt"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdPrefs() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "perfs",
		Short: "debug perfs",
		Run: func(cmd *cobra.Command, args []string) {
			clearExec := exec.Command("clear") //Linux example, its tested
			clearExec.Stdout = os.Stdout
			clearExec.Run()

			// Print content of ~/.fig/settings.json
			fmt.Println("~/.fig/settings.json:")
			settingsExec := exec.Command("cat", "~/.fig/settings.json")
			settingsExec.Stdout = os.Stdout
			settingsExec.Stderr = os.Stderr
			settingsExec.Run()

			// Print content of ~/.fig/user/config
			fmt.Println("~/.fig/user/config:")
			configExec := exec.Command("cat", "~/.fig/user/config")
			configExec.Stdout = os.Stdout
			configExec.Stderr = os.Stderr
			configExec.Run()

			// Print NSUserDefaults
			fmt.Println("NSUserDefaults:")
			userDefaultsExec := exec.Command("defaults", "read", "com.mschrage.fig")
			userDefaultsExec.Stdout = os.Stdout
			userDefaultsExec.Stderr = os.Stderr
			userDefaultsExec.Run()

			userDefaultsExecShared := exec.Command("defaults", "read", "com.mschrage.fig.shared")
			userDefaultsExecShared.Stdout = os.Stdout
			userDefaultsExecShared.Stderr = os.Stderr
			userDefaultsExecShared.Run()
		},
	}

	return cmd
}
