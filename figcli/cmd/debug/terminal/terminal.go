package terminal

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdTerminal() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "terminal",
		Short: "debug terminal",
		Run: func(cmd *cobra.Command, args []string) {

			clearExec := exec.Command("clear") //Linux example, its tested
			clearExec.Stdout = os.Stdout
			clearExec.Run()

			fmt.Println()
			fmt.Println()
			fmt.Println("===tty characteristics===")
			fmt.Println()

			sttyExec := exec.Command("stty", "-a")

			sttyExec.Stdout = os.Stdout
			sttyExec.Stderr = os.Stderr
			sttyExec.Stdin = os.Stdin

			sttyExec.Run()

			fmt.Println()
			fmt.Println()
			fmt.Print(" Press enter to contine...")

			bufio.NewReader(os.Stdin).ReadByte()

			fmt.Println()
			fmt.Println()
			fmt.Println("===environment vars===")
			fmt.Println()

			sttyExec = exec.Command("env")

			sttyExec.Stdout = os.Stdout
			sttyExec.Stderr = os.Stderr
			sttyExec.Stdin = os.Stdin

			sttyExec.Run()
		},
	}

	return cmd
}
