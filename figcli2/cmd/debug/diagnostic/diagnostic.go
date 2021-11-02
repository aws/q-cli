package diagnostic

import (
	"fig-cli/diagnostics"
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdDiagnostic() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "diagnostic",
		Short: "Run diagnostic tests",
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Println(diagnostics.Summary())
		},
	}

	return cmd
}
