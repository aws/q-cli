package cmd

import (
	"fmt"
	"io/ioutil"
	"os"

	"github.com/spf13/cobra"
)

const (
	versionNumber = "2"
)

func init() {
	callbackCmd.Flags().Bool("version", false, "Print version information")

	rootCmd.AddCommand(callbackCmd)
}

var callbackCmd = &cobra.Command{
	Use:    "callback",
	Hidden: true,
	Run: func(cmd *cobra.Command, args []string) {
		debug := os.Getenv("FIG_DEBUG") != ""

		if cmd.Flags().Lookup("version").Value.String() == "true" {
			fmt.Println(versionNumber)
			os.Exit(0)
		}

		if len(args) == 0 {
			if debug {
				fmt.Println("callback must include a handlerId")
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
				fmt.Println("No data on stdin!")
			}
			os.Exit(1)
		}

		// Get the handlerId and filename/exitcode if present
		handlerId := args[0]
		var filename string
		exitcode := "-1"

		if debug {
			fmt.Println("handlerId:", handlerId)
		}

		if len(args) == 3 {
			if debug {
				fmt.Printf("callback specified filepath (%s) and exitCode (%s) to output!\n", args[1], args[2])
			}

			filename = args[1]
			exitcode = args[2]
		} else {
			// Create tmp file
			file, err := ioutil.TempFile("/tmp/", "fig-callback-*")
			if err != nil {
				panic(err)
			}

			filename = file.Name()

			if debug {
				fmt.Printf("Created tmp file: %s\n", file.Name())
			}

			// Copy stdin to tmp file
			for {
				buf := make([]byte, 1024)
				n, err := stdin.Read(buf)
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
					fmt.Printf("Read %d bytes\n", size)
					fmt.Printf("%s\n", buf)
				}
			}

			if debug {
				fmt.Println("EOF!")
			}
		}

		// Send
		if debug {
			fmt.Println("Done reading from stdin!")
		}

		callback := fmt.Sprintf("fig pty:callback %s %s %s", handlerId, filename, exitcode)

		if debug {
			fmt.Printf("Sending '%s' over unix socket!\n", callback)
		}
	},
}
