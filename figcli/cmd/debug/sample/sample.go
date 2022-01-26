package sample

import (
	"fmt"
	"os/exec"
	"strings"

	"github.com/spf13/cobra"
)

func NewCmdSample() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sample",
		Short: "sample fig",
		Run: func(cmd *cobra.Command, args []string) {
			const outfile = "/tmp/fig-sample"

			execPid, err := exec.Command("lsappinfo", "info", "-only", "pid", "-app", "com.mschrage.fig").Output()
			execPidStr := strings.Split(strings.TrimSpace(string(execPid)), "=")[1]

			if err != nil {
				fmt.Println("Could not get Fig app pid")
				return
			}

			fmt.Printf("Sampling Fig process (%s). Writing output to %s\n", execPidStr, outfile)

			if err := exec.Command("sample", "-f", outfile, execPidStr).Run(); err != nil {
				fmt.Println("Could not sample Fig process")
				return
			}

			fmt.Printf("\n\n\n-------\nFinished writing to %s\n", outfile)
			fmt.Println("Please send this file to the Fig Team")
			fmt.Println("Or attach it to a GitHub issue (run 'fig issue')")
		},
	}

	return cmd
}
