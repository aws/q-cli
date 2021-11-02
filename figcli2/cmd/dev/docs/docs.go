package docs

import (
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdDocs() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "docs",
		Short: "documentation for building completion specs",
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Println("→ Opening docs in browser...")
			exec.Command("open", "https://fig.io/docs/getting-started").Run()
		},
	}

	return cmd
}
