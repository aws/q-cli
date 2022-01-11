package source

import (
	"fmt"

	"github.com/spf13/cobra"
)

func NewCmdSource() *cobra.Command {
	var shell string
	var login bool

	cmd := &cobra.Command{
		Use:   "source",
		Short: "Source dotfiles",
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Printf("# Source dotfiles for %v\n", shell)

			// TODO: Print the commands to source in the dotfiles
		},
	}

	cmd.Flags().StringVarP(&shell, "shell", "s", "", "shell to source")
	cmd.Flags().BoolVarP(&login, "login", "l", false, "login")

	return cmd
}
