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
	debugCmd.AddCommand(debugSshCmd)
	debugCmd.AddCommand(debugSampleCmd)
	debugCmd.AddCommand(debugDotfilesCmd)

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

var debugSshCmd = &cobra.Command{
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

var debugSampleCmd = &cobra.Command{
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

		if err := exec.Command("sample", "-p", execPidStr, "-o", outfile).Run(); err != nil {
			fmt.Println("Could not sample Fig process")
			return
		}

		fmt.Printf("\n\n\n-------\nFinished writing to %s\n", outfile)
		fmt.Println("Please send this file to the Fig Team")
		fmt.Println("Or attach it to a GitHub issue (run 'fig issue')")
	},
}

var debugDotfilesCmd = &cobra.Command{
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

var debugPerfsCmd = &cobra.Command{
	Use:   "perfs",
	Short: "debug perfs",
	Run: func(cmd *cobra.Command, args []string) {
		clearExec := exec.Command("clear") //Linux example, its tested
		clearExec.Stdout = os.Stdout
		clearExec.Run()

		// Print content of ~/.fig/settings.json
		fmt.Println("~/.fig/settings.json:")
		settingsExec := exec.Command("cat", "~/.fig/settings.json")
		settingsExec.Stdout = os.Stdout
		settingsExec.Stderr = os.Stderr
		settingsExec.Run()

		// Print content of ~/.fig/user/config
		fmt.Println("~/.fig/user/config:")
		configExec := exec.Command("cat", "~/.fig/user/config")
		configExec.Stdout = os.Stdout
		configExec.Stderr = os.Stderr
		configExec.Run()

		// Print NSUserDefaults
		fmt.Println("NSUserDefaults:")
		userDefaultsExec := exec.Command("defaults", "read", "com.mschrage.fig")
		userDefaultsExec.Stdout = os.Stdout
		userDefaultsExec.Stderr = os.Stderr
		userDefaultsExec.Run()

		userDefaultsExecShared := exec.Command("defaults", "read", "com.mschrage.fig.shared")
		userDefaultsExecShared.Stdout = os.Stdout
		userDefaultsExecShared.Stderr = os.Stderr
		userDefaultsExecShared.Run()
	},
}
