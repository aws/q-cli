package contrib

import (
	"fig-cli/cmd/specs/list"
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdContrib() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "contrib",
		Short: "Contribute to Fig CLI",
		Long:  "Contribute to Fig CLI",
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Println("→ Opening GitHub repo...")
			exec.Command("open", "https://github.com/withfig/autocomplete").Run()
		},
	}

	cmd.AddCommand(list.NewCmdList())

	return cmd
}
