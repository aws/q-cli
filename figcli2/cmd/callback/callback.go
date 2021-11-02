package callback

import (
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fmt"
	"io"
	"os"

	"github.com/spf13/cobra"
)

const (
	versionNumber = "2"
)

func init() {
	callbackCmd.Flags().Bool("version", false, "Print version information")
}

var callbackCmd = &cobra.Command{
	Use:    "callback",
	Hidden: true,
	Run: func(cmd *cobra.Command, args []string) {
		output := os.Stderr
		debug := os.Getenv("FIG_DEBUG") != ""

		if debug {
			fmt.Fprintln(output, "Fig CLI version:", versionNumber)
		}

		if cmd.Flags().Lookup("version").Value.String() == "true" {
			fmt.Fprintln(output, versionNumber)
			os.Exit(0)
		}

		if len(args) == 0 {
			if debug {
				fmt.Fprintln(output, "callback must include a handlerId")
			}
			os.Exit(1)
		}

		// Check if data is on stdin
		stdin := os.Stdin
		fi, err := stdin.Stat()
		if err != nil {
			panic(err)
		}
		size := fi.Size()
		if size == 0 {
			if debug {
				fmt.Fprintln(output, "No data on stdin!")
			}
			os.Exit(1)
		}

		// Get the handlerId and filename/exitcode if present
		handlerId := args[0]
		var filename string
		exitcode := "-1"

		if debug {
			fmt.Fprintln(output, "handlerId:", handlerId)
		}

		if len(args) == 3 {
			if debug {
				fmt.Fprintf(output, "callback specified filepath (%s) and exitCode (%s) to output!\n", args[1], args[2])
			}

			filename = args[1]
			exitcode = args[2]
		} else {
			// Create tmp file
			file, err := os.CreateTemp("/tmp/", "fig-callback-*")
			if err != nil {
				panic(err)
			}

			filename = file.Name()

			if debug {
				fmt.Fprintln(output, "Created tmp file:", file.Name())
			}

			// Copy stdin to tmp file
			for {
				buf := make([]byte, 1024)
				n, err := stdin.Read(buf)
				if err == io.EOF {
					break
				}

				if err != nil {
					panic(err)
				}

				// If we read 0 bytes, we're done
				if n == 0 {
					break
				}

				// Write to tmp file
				file.Write(buf[:n])

				if debug {
					fmt.Fprintf(output, "Read %d bytes\n", size)
					fmt.Fprintf(output, "%s\n", buf)
				}
			}

			if debug {
				fmt.Fprintln(output, "EOF!")
			}
		}

		// Send
		if debug {
			fmt.Fprintln(output, "Done reading from stdin!")
		}

		callback := fmt.Sprintf("fig pty:callback %s %s %s", handlerId, filename, exitcode)

		if debug {
			fmt.Fprintf(output, "Sending '%s' over unix socket!\n", callback)
		}

		callbackHook := fig_proto.Hook{
			Hook: &fig_proto.Hook_Callback{
				Callback: &fig_proto.CallbackHook{
					HandlerId: handlerId,
					Filepath:  filename,
					ExitCode:  exitcode,
				},
			},
		}

		err = fig_ipc.SendHook(&callbackHook)
		if debug && err != nil {
			fmt.Fprintln(output, "Error sending callback:", err)
		}
	},
}
