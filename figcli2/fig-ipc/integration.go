package fig_ipc

import (
	fig_proto "fig-cli/fig-proto"
)

type Integration string

const (
	IntegrationIterm          Integration = "com.googlecode.iterm2"
	IntegrationTerminal       Integration = "com.apple.Terminal"
	IntegrationHyper          Integration = "co.zeit.hyper"
	IntegrationVSCode         Integration = "com.microsoft.VSCode"
	IntegrationVSCodeInsiders Integration = "com.microsoft.VSCodeInsiders"
	IntegrationVSCodium       Integration = "com.visualstudio.code.oss"
	IntegrationKitty          Integration = "net.kovidgoyal.kitty"
	IntegrationAlacritty      Integration = "io.alacritty"
)

func CreateTerminalIntegrationRequest(
	identifier Integration,
	action fig_proto.IntegrationAction,
) *fig_proto.Command {
	id := int64(0)
	noResponse := false

	return &fig_proto.Command{
		Id:         &id,
		NoResponse: &noResponse,
		Command: &fig_proto.Command_TerminalIntegrationUpdate{
			TerminalIntegrationUpdate: &fig_proto.TerminalIntegrationRequest{
				Identifier: string(identifier),
				Action:     action,
			},
		},
	}
}

func CreateListTerminalIntegrations() *fig_proto.Command {
	id := int64(0)
	noResponse := false

	return &fig_proto.Command{
		Id:         &id,
		NoResponse: &noResponse,
		Command: &fig_proto.Command_ListTerminalIntegrations{
			ListTerminalIntegrations: &fig_proto.ListTerminalIntegrations{},
		},
	}
}

func GetIntegrations() (interface{}, error) {
	terminalIntegrationRequest := CreateListTerminalIntegrations()
	res, err := SendRecvCommand(terminalIntegrationRequest)
	return res, err
}

func IntegrationInstall(integration Integration) (string, error) {
	terminalIntegrationRequest := CreateTerminalIntegrationRequest(integration, fig_proto.IntegrationAction_INSTALL)
	res, err := SendRecvCommand(terminalIntegrationRequest)
	return res.(string), err
}

func IntegrationVerifyInstall(integration Integration) (string, error) {
	terminalIntegrationRequest := CreateTerminalIntegrationRequest(integration, fig_proto.IntegrationAction_VERIFY_INSTALL)
	res, err := SendRecvCommand(terminalIntegrationRequest)
	return res.(string), err
}

func IntegrationUninstall(integration Integration) (string, error) {
	terminalIntegrationRequest := CreateTerminalIntegrationRequest(integration, fig_proto.IntegrationAction_UNINSTALL)
	res, err := SendRecvCommand(terminalIntegrationRequest)
	return res.(string), err
}
