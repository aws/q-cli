package cmd

import (
	"encoding/json"
	"fig-cli/settings"
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func init() {
	settingsCmd.Flags().Bool("delete", false, "Delete the key")
	settingsCmd.AddCommand(settingsDocsCmd)
	settingsCmd.AddCommand(settingsOpenCmd)

	rootCmd.AddCommand(settingsCmd)
}

var settingsCmd = &cobra.Command{
	Use:   "settings [key] [value]",
	Short: "Get or set a setting",
	Long:  "Get or set a setting",
	Args:  cobra.RangeArgs(0, 2),
	Annotations: map[string]string{
		"figcli.command.categories":      "Common",
		"figcli.command.argDescriptions": "[key] key to get or set\n[value] value to set (optional)",
	},
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

var settingsDocsCmd = &cobra.Command{
	Use:   "docs",
	Short: "Get the settings documentation",
	Long:  "Get the settings documentation",
	Run: func(cmd *cobra.Command, arg []string) {
		exec.Command("open", "https://fig.io/docs/support/settings").Run()
	},
}

var settingsOpenCmd = &cobra.Command{
	Use:   "open",
	Short: "Open the settings file",
	Long:  "Open the settings file",
	Run: func(cmd *cobra.Command, arg []string) {
		settingsFilepath, err := settings.GetFilepath()
		if err != nil {
			fmt.Println(err)
			return
		}

		if err := exec.Command("open", settingsFilepath).Run(); err != nil {
			fmt.Println(err)
		}
	},
}
