package cmd

import (
	"encoding/json"
	"fig-cli/specs"
	"fmt"
	"net/http"
	"os"
	"os/user"
	"strings"

	"github.com/spf13/cobra"
)

func init() {
	specsListCmd.Flags().Bool("remote", false, "List specs that are available on the remote")

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
		if cmd.Flags().Lookup("remote").Value.String() == "true" {
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
			specs, _ := specs.GetSpecsNames()
			for _, spec := range specs {
				fmt.Println(spec)
			}
		}

	},
}
