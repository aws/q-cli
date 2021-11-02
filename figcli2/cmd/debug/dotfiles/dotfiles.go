package dotfiles

import (
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdDotfiles() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "dotfiles",
		Short: "debug dotfiles",
		Run: func(cmd *cobra.Command, args []string) {
			// TODO: Replace with native implementation
			sh := exec.Command("bash", "-c", "~/.fig/tools/cli/email_dotfiles.sh")
			sh.Stdout = os.Stdout
			sh.Stderr = os.Stderr
			sh.Stdin = os.Stdin
			sh.Run()
		},
	}

	return cmd
}
