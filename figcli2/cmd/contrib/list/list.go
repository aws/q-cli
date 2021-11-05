package list

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"github.com/spf13/cobra"
)

func NewCmdList() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "contributors",
		Short: "List the contributors to Fig",
		Long:  "List the contributors to Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			url := "https://api.github.com/repos/withfig/autocomplete/contributors?per_page=100"

			resp, err := http.Get(url)
			if err != nil {
				fmt.Println("Unable to get contributors")
				return
			}

			defer resp.Body.Close()
			body, err := io.ReadAll(resp.Body)
			if err != nil {
				fmt.Println("Unable to read contributors")
				return
			}

			var contributors []interface{}
			err = json.Unmarshal(body, &contributors)
			if err != nil {
				fmt.Println("Unable to parse contributors")
				return
			}

			fmt.Println("Contributors:")
			fmt.Println("------------")
			for _, contributor := range contributors {
				contributorMap := contributor.(map[string]interface{})
				login := contributorMap["login"].(string)
				url := contributorMap["html_url"].(string)
				contribs := contributorMap["contributions"].(float64)
				fmt.Printf("%.0f - %s - %s\n", contribs, login, url)
			}

		},
	}

	return cmd
}
