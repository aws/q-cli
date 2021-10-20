package cmd

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"strings"

	"github.com/spf13/cobra"
)

func init() {
	debugCmd.AddCommand(debugLogsCmd)
	debugCmd.AddCommand(debugAppCmd)
	debugCmd.AddCommand(debugTerminalCmd)
	debugCmd.AddCommand(debugUnixSocketCmd)
	debugCmd.AddCommand(debugVerifyCodesignCmd)

	rootCmd.AddCommand(debugCmd)
}

var debugCmd = &cobra.Command{
	Use:   "debug",
	Short: "debug fig",
}

var debugLogsCmd = &cobra.Command{
	Use:   "logs",
	Short: "debug fig logs",
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

		// Tail logs
		tailExec := exec.Command("sh", "-c", "tail -n0 -qf ~/.fig/logs/*.log")

		tailExec.Stdout = os.Stdout
		tailExec.Stderr = os.Stderr

		tailExec.Run()
	},
}

var debugAppCmd = &cobra.Command{
	Use:   "app",
	Short: "debug fig app",
	Run: func(cmd *cobra.Command, args []string) {
		running, err := exec.Command("fig", "app:running").Output()
		runningStr := strings.TrimSpace(string(running))

		if err != nil {
			fmt.Println("Could not get running app")
			return
		}

		if runningStr == "0" {
			fmt.Println("Fig app is not currently running...")

			execFig := exec.Command("/Applications/Fig.app/Contents/MacOS/fig")
			err := execFig.Start()

			if err != nil {
				fmt.Println("Could not start fig")
			}

			execFig.Process.Release()
			return
		}

		bundelPath, err := exec.Command("lsappinfo", "info", "-only", "bundlepath", "-app", "com.mschrage.fig").Output()
		bundelPathStr := strings.Replace(strings.Split(strings.TrimSpace(string(bundelPath)), "=")[1], "\"", "", -1)

		if err != nil {
			fmt.Println("Could not get Fig app bundle path")
			return
		}

		front, err := exec.Command("lsappinfo", "front").Output()
		frontStr := strings.TrimSpace(string(front))

		if err != nil {
			fmt.Println("Could not get front app")
			return
		}

		terminalEmu, err := exec.Command("lsappinfo", "info", "-only", "name", "-app", frontStr).Output()
		terminalEmuStr := strings.Replace(strings.Split(strings.TrimSpace(string(terminalEmu)), "=")[1], "\"", "", -1)

		if err != nil {
			fmt.Println("Could not get terminal emulator app")
			return
		}

		fmt.Println("Running the Fig.app executable directly from " + bundelPathStr + ".")
		fmt.Println("You will need to grant accessibility permissions to the current terminal (" + terminalEmuStr + ")!")
	},
}

var debugTerminalCmd = &cobra.Command{
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

var debugUnixSocketCmd = &cobra.Command{
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

var debugVerifyCodesignCmd = &cobra.Command{
	Use:   "verify-codesign",
	Short: "debug fig verify-codesign",
	Run: func(cmd *cobra.Command, args []string) {
		codesignExec := exec.Command("codesign", "-vvvv", "/Applications/Fig.app")

		codesignExec.Stdout = os.Stdout
		codesignExec.Stderr = os.Stderr

		codesignExec.Run()
	},
}
