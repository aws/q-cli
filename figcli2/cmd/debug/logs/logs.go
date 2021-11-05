package logs

import (
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"strings"

	"github.com/spf13/cobra"
)

func NewCmdLogs() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "logs",
		Short: "debug fig logs",
		Args:  cobra.ArbitraryArgs,
		Run: func(cmd *cobra.Command, args []string) {
			// Start logging
			loggingExec := exec.Command("fig", "settings", "developer.logging", "true")

			if err := loggingExec.Run(); err != nil {
				fmt.Println("Could not start logging")
				return
			}

			// Capture ctrl^c and stop logging
			c := make(chan os.Signal, 1)
			signal.Notify(c, os.Interrupt)
			go func() {
				<-c
				exec.Command("fig", "settings", "developer.logging", "false").Run()
				os.Exit(0)
			}()

			files := []string{}

			if len(args) > 0 {
				for _, arg := range args {
					file := fmt.Sprintf("~/.fig/logs/%s.log", arg)
					files = append(files, file)
				}
			} else {
				files = append(files, "~/.fig/logs/*.log")
			}

			command := fmt.Sprintf("tail -n0 -qf %s", strings.Join(files, " "))

			// Tail logs
			tailExec := exec.Command("sh", "-c", command)

			tailExec.Stdout = os.Stdout
			tailExec.Stderr = os.Stderr

			tailExec.Run()
		},
	}

	return cmd
}
