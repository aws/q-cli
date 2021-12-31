package unixsocket

import (
	"fmt"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdUnixSocket() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "unix-socket",
		Short: "debug unix socket",
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println("Listening on /tmp/fig.socket...")
			fmt.Println("Note: You will need to restart Fig afterwards")

			// Delete old socket
			os.Remove("/tmp/fig.socket")

			// Run nc
			ncExec := exec.Command("nc", "-Ulk", "/tmp/fig.socket")
			ncExec.Stdout = os.Stdout
			ncExec.Stderr = os.Stderr
			ncExec.Stdin = os.Stdin
			ncExec.Run()
		},
	}

	return cmd
}
