package cmd

import (
	"encoding/json"
	"fig-cli/settings"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	settingsCmd.Flags().Bool("delete", false, "Delete the key")

	rootCmd.AddCommand(settingsCmd)
}

var settingsCmd = &cobra.Command{
	Use:   "settings [key] [value]",
	Short: "documentation for building completion specs",
	Args:  cobra.ArbitraryArgs,
	Run: func(cmd *cobra.Command, arg []string) {
		result, err := settings.Load()
		if err != nil {
			fmt.Println(err)
			return
		}

		// If flag is set, delete the key
		if cmd.Flag("delete").Value.String() == "true" {
			result.Delete(arg[0])
			result.Save()
			return
		}

		if len(arg) == 1 {
			val := result[arg[0]]
			if val != nil {
				fmt.Println(result[arg[0]])
			} else {
				fmt.Println("No value associated with '" + arg[0] + "'.")
			}
		}

		if len(arg) >= 2 {
			val := arg[1]

			var jsonVal interface{}
			err = json.Unmarshal([]byte(val), &jsonVal)

			if err == nil {
				result.Set(arg[0], jsonVal)
			} else {
				result.Set(arg[0], val)
			}

			result.Save()
		}
	},
}
