package unixsocket

import (
	"fmt"
	"os"
	"os/exec"

	fig_ipc "fig-cli/fig-ipc"

	"github.com/spf13/cobra"
)

func NewCmdUnixSocket() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "unix-socket",
		Short: "debug unix socket",
		Run: func(cmd *cobra.Command, args []string) {
			socket := fig_ipc.GetSocketPath()
			fmt.Println("Listening on", socket, "...")
			fmt.Println("Note: You will need to restart Fig afterwards")

			// Delete old socket
			os.Remove(socket)

			// Run nc
			ncExec := exec.Command("nc", "-Ulk", socket)
			ncExec.Stdout = os.Stdout
			ncExec.Stderr = os.Stderr
			ncExec.Stdin = os.Stdin
			ncExec.Run()
		},
	}

	return cmd
}
