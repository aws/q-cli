package ssh

import (
	"fmt"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdSsh() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "ssh",
		Short: "debug ssh",
		Run: func(cmd *cobra.Command, args []string) {
			execSsh := exec.Command("ssh", "-V")
			execSsh.Stdout = os.Stdout
			execSsh.Stderr = os.Stderr
			execSsh.Run()

			fmt.Println("~/.ssh/config:")

			configExec := exec.Command("cat", "~/.ssh/config")
			configExec.Stdout = os.Stdout
			configExec.Stderr = os.Stderr
			configExec.Run()
		},
	}

	return cmd
}
