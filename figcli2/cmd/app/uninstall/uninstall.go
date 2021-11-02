package uninstall

import (
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCommandUninstall() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "uninstall",
		Short: "Uninstall Fig",
		Long:  `Uninstall Fig`,
		Run: func(cmd *cobra.Command, args []string) {
			sh := exec.Command("bash", "-c", "~/.fig/tools/uninstall-script.sh")
			sh.Stdout = os.Stdout
			sh.Stderr = os.Stderr
			sh.Stdin = os.Stdin
			sh.Run()
		},
	}

	return cmd
}
