package dev

import (
	"fig-cli/cmd/dev/docs"

	"github.com/spf13/cobra"
)

func NewCmdDev() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "dev",
		Short: "dev commands",
	}

	cmd.AddCommand(docs.NewCmdDocs())

	return cmd
}
