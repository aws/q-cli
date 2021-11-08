package list

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/spf13/cobra"
)

func NewCmdList() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "list",
		Short: "List autcomplete specs",
		Run: func(cmd *cobra.Command, arg []string) {
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

			// if cmd.Flags().Lookup("local").Value.String() == "true" {
			// 	specs, _ := specs.GetSpecsNames()
			// 	for _, spec := range specs {
			// 		fmt.Println(spec)
			// 	}
			// }
		},
	}

	return cmd
}
