package main

import (
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fig-cli/logging"
	"fmt"
	"io"
	"os"
	"path/filepath"
)

const (
	versionNumber = "3"
)

func main() {
	output := os.Stderr
	debug := true //os.Getenv("FIG_DEBUG") != ""

	if len(os.Args) <= 2 {
		if debug {
			fmt.Fprintln(output, "callback must include a handlerId")
			logging.Log("callback must include a handlerId")
		}
		os.Exit(1)
	}

	args := os.Args[2:]

	if debug {
		logging.Log("Fig callback version", versionNumber)
		// fmt.Fprintln(output, "Fig CLI version:", versionNumber)
	}

	// Check if data is on stdin
	stdin := os.Stdin
	// fi, err := stdin.Stat()
	// if err != nil {
	// 	panic(err)
	// }

	// size := fi.Size()
	// if size == 0 {
	// 	if debug {
	// 		fmt.Fprintln(output, "No data on stdin!")
	// 		logging.Log("No data on stdin!")
	// 	}
	// 	os.Exit(1)
	// }

	// Get the handlerId and filename/exitcode if present
	handlerId := args[0]
	filename := ""
	exitcode := "-1"

	if debug {
		fmt.Fprintln(output, "handlerId:", handlerId)
		logging.Log("handlerId:", handlerId)
	}

	if len(args) == 3 {
		if debug {
			fmt.Fprintf(output, "callback specified filepath (%s) and exitCode (%s) to output!\n", args[1], args[2])
			logging.Log(fmt.Sprintf("callback specified filepath (%s) and exitCode (%s) to output!", args[1], args[2]))
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
			logging.Log("Created tmp file:", file.Name())
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
				fmt.Fprintf(output, "Read %d bytes\n", n)
				logging.Log(fmt.Sprintf("Read %d bytes", n))
				fmt.Fprintf(output, "%s\n", buf)
				logging.Log(fmt.Sprintf("%s", buf))
			}
		}

		if debug {
			fmt.Fprintln(output, "EOF!")
		}
	}

	// Send
	if debug {
		fmt.Fprintln(output, "Done reading from stdin!")
		logging.Log("Done reading from stdin!")
	}

	callback := fmt.Sprintf("handledId: %s, filename: %s, exitcode: %s", handlerId, filename, exitcode)

	if debug {
		fmt.Fprintf(output, "Sending '%s' over unix socket!\n", callback)
		logging.Log(fmt.Sprintf("Sending '%s' over unix socket!", callback))
		logging.Log(filepath.Join(os.TempDir(), "fig.socket"))
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

	err := fig_ipc.SendHook(&callbackHook)
	if debug && err != nil {
		fmt.Fprintln(output, "Error sending callback:", err)
		logging.Log("Error sending callback:", err.Error())
	} else if debug {
		logging.Log("Callback sent")
	}
}
