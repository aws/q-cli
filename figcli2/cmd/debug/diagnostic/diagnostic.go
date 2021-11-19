package diagnostic

import (
	"fig-cli/diagnostics"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdDiagnostic() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "diagnostic",
		Aliases: []string{"diagnostics"},
		Short:   "Run diagnostic tests",
		Long:    `Run diagnostic tests`,
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Println(diagnostics.Summary())
		},
	}

	return cmd
}
