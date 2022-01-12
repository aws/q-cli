package alpha

import (
	"fig-cli/cmd/alpha/source"
	"fmt"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdAlpha() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "alpha",
		Short: "Open mission control",
		Run: func(cmd *cobra.Command, args []string) {
			// TODO: send protobuf to open local mission control
			if err := exec.Command("open", "https://fig.io").Run(); err != nil {
				fmt.Printf("Error opening mission control: %s", err)
				os.Exit(1)
			}

			fmt.Printf("\n→ Opening mission control\n\n")
		},
	}

	cmd.AddCommand(source.NewCmdSource())

	return cmd
}
