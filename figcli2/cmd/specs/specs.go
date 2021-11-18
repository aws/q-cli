package specs

import (
	"fig-cli/cmd/specs/list"

	"github.com/spf13/cobra"
)

func NewCmdSpecs() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "specs",
		Short: "Manage your specs",
		Long:  `Manage your autocomplete specs`,
		// Annotations: map[string]string{
		// 	"figcli.command.categories": "Common",
		// },
	}

	cmd.AddCommand(list.NewCmdList())

	return cmd
}
