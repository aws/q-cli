package verifycodesign

import (
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdVerifyCodesign() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "verify-codesign",
		Short: "debug fig verify-codesign",
		Run: func(cmd *cobra.Command, args []string) {
			codesignExec := exec.Command("codesign", "-vvvv", "/Applications/Fig.app")

			codesignExec.Stdout = os.Stdout
			codesignExec.Stderr = os.Stderr

			codesignExec.Run()
		},
	}

	return cmd
}
