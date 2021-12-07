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
			fmt.Printf("\n→ Opening docs in browser...\n\n")
			if err := exec.Command("open", "https://fig.io/docs/").Run(); err != nil {
				fmt.Println(err)
			}
		},
	}

	return cmd
}
