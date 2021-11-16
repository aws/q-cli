package contrib

import (
	"fig-cli/cmd/contrib/list"
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
			fmt.Printf("\n→ Opening GitHub repo...\n\n")
			if err := exec.Command("open", "https://github.com/withfig/autocomplete").Run(); err != nil {
				fmt.Println(err)
			}
		},
	}

	cmd.AddCommand(list.NewCmdList())

	return cmd
}
