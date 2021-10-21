package cmd

import (
	fig_ipc "fig-cli/fig-ipc"
	"fmt"

	"github.com/spf13/cobra"
)

var integrationList = []string{
	"iterm",
	"hyper",
	"vscode",
	"terminal",
	"alacritty",
}

var integrationMap = map[string]fig_ipc.Integration{
	"iterm":     fig_ipc.IntegrationIterm,
	"hyper":     fig_ipc.IntegrationHyper,
	"vscode":    fig_ipc.IntegrationVSCode,
	"terminal":  fig_ipc.IntegrationTerminal,
	"alacritty": fig_ipc.IntegrationAlacritty,
}

func init() {
	integrationsCmd.AddCommand(integrationsListCmd)
	integrationsCmd.AddCommand(integrationsInstallCmd)
	integrationsCmd.AddCommand(integrationsUninstallCmd)
	integrationsCmd.AddCommand(integrationsVerifyCmd)

	rootCmd.AddCommand(integrationsCmd)
}

var integrationsCmd = &cobra.Command{
	Use:   "integrations",
	Short: "Manage Fig integrations",
	Long:  "Install, uninstall, and verify Fig integrations",
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},
}

var integrationsListCmd = &cobra.Command{
	Use:   "list",
	Short: "List Fig integrations",
	Long:  "List Fig integrations",
	Run: func(cmd *cobra.Command, args []string) {
		res, err := fig_ipc.GetIntegrations()
		fmt.Println(res, err)
	},
}

var integrationsInstallCmd = &cobra.Command{
	Use:       "install [integration]",
	Short:     "Install Fig integrations",
	Long:      "Install Fig integrations",
	Args:      cobra.ExactValidArgs(1),
	ValidArgs: integrationList,
	Run: func(cmd *cobra.Command, args []string) {
		integration := args[0]
		if _, err := fig_ipc.IntegrationInstall(integrationMap[integration]); err != nil {
			fmt.Printf("Error installing integration: %s\n", err.Error())
		} else {
			fmt.Printf("Integration %s installed\n", integration)
		}
	},
}

var integrationsUninstallCmd = &cobra.Command{
	Use:       "uninstall [integration]",
	Short:     "Uninstall Fig integrations",
	Long:      "Uninstall Fig integrations",
	Args:      cobra.ExactValidArgs(1),
	ValidArgs: integrationList,
	Run: func(cmd *cobra.Command, args []string) {
		integration := args[0]
		if _, err := fig_ipc.IntegrationUninstall(integrationMap[integration]); err != nil {
			fmt.Printf("Error uninstalling integration: %s\n", err.Error())
		} else {
			fmt.Printf("Integration %s uninstalled\n", integration)
		}
	},
}

var integrationsVerifyCmd = &cobra.Command{
	Use:       "verify [integration]",
	Short:     "Verify Fig integrations",
	Long:      "Verify Fig integrations",
	Args:      cobra.ExactValidArgs(1),
	ValidArgs: integrationList,
	Run: func(cmd *cobra.Command, args []string) {
		integration := args[0]
		if _, err := fig_ipc.IntegrationVerifyInstall(integrationMap[integration]); err != nil {
			fmt.Printf("Error verifying integration: %s\n", err.Error())
		} else {
			fmt.Printf("Integration %s is verified\n", integration)
		}
	},
}
