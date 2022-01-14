package open

import (
	"fig-cli/settings"
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdOpen() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "open",
		Short: "Open the settings file",
		Long:  "Open the settings file",
		Run: func(cmd *cobra.Command, arg []string) {
			settingsFilepath, err := settings.GetFilepath()
			if err != nil {
				fmt.Println(err)
				return
			}

			if err := exec.Command("open", settingsFilepath).Run(); err != nil {
				fmt.Println(err)
			}
		},
	}

	return cmd
}
