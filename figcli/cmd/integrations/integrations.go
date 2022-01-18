package integrations

import (
	fig_ipc "fig-cli/fig-ipc"

	"github.com/spf13/cobra"
)

var IntegrationList = []string{
	"iterm",
	"hyper",
	"vscode",
	"terminal",
	"alacritty",
}

var IntegrationMap = map[string]fig_ipc.Integration{
	"iterm":     fig_ipc.IntegrationIterm,
	"hyper":     fig_ipc.IntegrationHyper,
	"vscode":    fig_ipc.IntegrationVSCode,
	"terminal":  fig_ipc.IntegrationTerminal,
	"alacritty": fig_ipc.IntegrationAlacritty,
}

func NewCmdIntegrations() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "integrations",
		Short: "Manage integrations",
		Long:  "Install, uninstall, and verify Fig integrations",
	}

	return cmd
}
