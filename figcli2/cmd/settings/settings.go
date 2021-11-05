package settings

import (
	"encoding/json"
	"fig-cli/cmd/settings/docs"
	"fig-cli/cmd/settings/open"
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fig-cli/settings"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdSettings() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "settings [key] [value]",
		Short: "Get or set a setting",
		Long:  "Get or set a setting",
		Args:  cobra.RangeArgs(0, 2),
		Annotations: map[string]string{
			"figcli.command.categories":      "Common",
			"figcli.command.argDescriptions": "[key] key to get or set\n[value] value to set (optional)",
		},
		Run: func(cmd *cobra.Command, arg []string) {
			if len(arg) == 0 {
				response, err := fig_ipc.RunOpenUiElementCommand(fig_proto.UiElement_SETTINGS)
				if err != nil {
					fmt.Println(err)
					return
				}

				if response != "" {
					fmt.Printf("\n%s\n\n", response)
				}
				return
			}

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

	cmd.Flags().BoolP("delete", "d", false, "delete the key")

	cmd.AddCommand(docs.NewCmdDocs())
	cmd.AddCommand(open.NewCmdOpen())
	cmd.AddCommand()

	return cmd
}
