package cmd

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"os/user"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
)

func init() {
	specsListCmd.Flags().Bool("local", false, "List local specs only")

	specsCmd.AddCommand(specsUninstallCmd)
	specsCmd.AddCommand(specsListCmd)

	rootCmd.AddCommand(specsCmd)
}

var specsCmd = &cobra.Command{
	Use:   "specs",
	Short: "Manage your specs",
	Long:  `Manage your autocomplete specs`,
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},
}

var specsUninstallCmd = &cobra.Command{
	Use:   "uninstall [spec]",
	Short: "Uninstall a spec",
	Args:  cobra.MinimumNArgs(1),
	Run: func(cmd *cobra.Command, arg []string) {
		// Get user
		user, err := user.Current()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		for _, spec := range arg {
			// Rm spec from ~/.fig/autocomplete
			if err = os.Remove(fmt.Sprintf("%s/.fig/autocomplete/%s.js", user.HomeDir, spec)); err != nil {
				fmt.Println("Unable to uninstall", spec)
			} else {
				fmt.Println("Uninstalled", spec)
			}
		}
	},
}

var specsListCmd = &cobra.Command{
	Use:   "list",
	Short: "List installed specs",
	Run: func(cmd *cobra.Command, arg []string) {
		// Get user
		user, err := user.Current()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		if cmd.Flags().Lookup("local").Value.String() == "false" {
			// Get remote specs
			req, _ := http.NewRequest("GET", "https://api.github.com/repos/withfig/autocomplete/contents/src", nil)
			req.Header.Set("Accept", "application/vnd.github.v3+json")

			client := &http.Client{}
			resp, err := client.Do(req)
			if err != nil {
				fmt.Println("Error:", err)
				return
			}

			defer resp.Body.Close()

			// Get spec files
			var specFiles []interface{}
			if err = json.NewDecoder(resp.Body).Decode(&specFiles); err != nil {
				fmt.Println("Error:", err)
				return
			}

			for _, specFile := range specFiles {
				spec := strings.TrimSuffix(strings.TrimPrefix(specFile.(map[string]interface{})["name"].(string), "autocomplete/src/"), ".ts")
				fmt.Println(spec)
			}
		} else {
			// Get specs
			files, err := filepath.Glob(fmt.Sprintf("%s/.fig/autocomplete/*.js", user.HomeDir))
			if err != nil {
				fmt.Println("Error:", err)
				return
			}

			// Print specs
			for _, file := range files {
				fileName := strings.TrimSuffix(strings.TrimPrefix(file, fmt.Sprintf("%s/.fig/autocomplete/", user.HomeDir)), ".js")
				fmt.Println(fileName)
			}
		}

	},
}
