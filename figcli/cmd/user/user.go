package user

import (
	"fig-cli/cmd/user/logout"
	"fig-cli/cmd/user/whoami"

	"github.com/spf13/cobra"
)

func NewCmdUser() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "user",
		Short: "user commands",
	}

	cmd.AddCommand(whoami.NewCmdWhoami())
	cmd.AddCommand(logout.NewCmdLogout())

	return cmd
}
