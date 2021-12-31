package onboarding

import (
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdOnboarding() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "onboarding",
		Short: "Run through onboarding process",
		Run: func(cmd *cobra.Command, arg []string) {
			sh := exec.Command("bash", "-c", "~/.fig/tools/drip/fig_onboarding.sh")
			sh.Stdout = os.Stdout
			sh.Stderr = os.Stderr
			sh.Stdin = os.Stdin
			sh.Run()
		},
	}

	return cmd
}
