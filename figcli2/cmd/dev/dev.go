package dev

import (
	"fig-cli/cmd/dev/build"
	"fig-cli/cmd/settings/docs"

	"github.com/spf13/cobra"
)

func NewCmdDev() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "dev",
		Short: "dev commands",
	}

	cmd.AddCommand(docs.NewCmdDocs())
	cmd.AddCommand(build.NewCmdBuild())

	return cmd
}
