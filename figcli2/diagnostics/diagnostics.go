package diagnostics

import (
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fig-cli/settings"
	"fig-cli/specs"
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"os/user"
	"regexp"
	"strings"
)

func GetMacOsVersion() (string, error) {
	execSwVers, err := exec.Command("sw_vers").Output()
	if err != nil {
		return "", err
	}

	regexpVersion := regexp.MustCompile(`ProductVersion:\s*(\S+)`)
	regexpBuild := regexp.MustCompile(`BuildVersion:\s*(\S+)`)

	version := regexpVersion.FindSubmatch(execSwVers)
	build := regexpBuild.FindSubmatch(execSwVers)

	if len(version) < 2 || len(build) < 2 {
		return "", nil
	}

	return string(version[1]) + "." + string(build[1]), nil
}

func ReadPlist(field string) (string, error) {
	plistData, err := ioutil.ReadFile("/Applications/Fig.app/Contents/Info.plist")
	if err != nil {
		return "", err
	}

	re, err := regexp.Compile(fmt.Sprintf("<key>%s</key>\\s*<\\S+>(\\S+)</\\S+>", field))
	if err != nil {
		return "", err
	}

	matches := re.FindStringSubmatch(string(plistData))
	if len(matches) == 2 {
		return matches[1], nil
	}

	return "unknown", fmt.Errorf("could not find field: %s", field)
}

func GetFigVersion() (string, error) {
	return ReadPlist("CFBundleShortVersionString")
}

func GetFigBuild() (string, error) {
	return ReadPlist("CFBundleVersion")
}

func DsclRead(value string) (string, error) {
	user, err := user.Current()
	if err != nil {
		return "unknown", err
	}

	execUserShell, err := exec.Command("dscl", ".", "-read", user.HomeDir, value).Output()
	if err != nil {
		return "", err
	}

	return strings.TrimSpace(string(execUserShell)), nil
}

type Terminal string

func GetCurrentTerminal() (Terminal, error) {
	if os.Getenv("KITTY_WINDOW_ID") != "" {
		return Terminal("kitty"), nil
	}

	if os.Getenv("ALACRITTY_LOG") != "" {
		return Terminal("alacritty"), nil
	}

	if strings.Contains(os.Getenv("TERM_PROGRAM_VERSION"), "insider") {
		return Terminal("vscode-insiders"), nil
	}

	term := os.Getenv("TERM_PROGRAM")
	if term == "" {
		return Terminal("unknown"), fmt.Errorf("could not determine terminal")
	}

	return Terminal(term), nil
}

func (t Terminal) PotentialBundleId() (string, error) {
	switch t {
	case Terminal("vscode-insiders"):
		return "com.microsoft.VSCodeInsiders", nil
	case Terminal("vscode"):
		return "com.microsoft.VSCode", nil
	case Terminal("Apple_Terminal"):
		return "com.apple.Terminal", nil
	case Terminal("Hyper"):
		return "co.zeit.hyper", nil
	case Terminal("iTerm.app"):
		return "com.googlecode.iterm2", nil
	}

	termBundle := os.Getenv("TERM_BUNDLE_IDENTIFIER")

	if termBundle == "" {
		return "unknown", fmt.Errorf("could not determine terminal bundle")
	}

	return termBundle, nil
}

func GetTty() (string, error) {
	ttyExec := exec.Command("tty")
	ttyExec.Stdin = os.Stdin

	out, err := ttyExec.Output()
	if err != nil {
		return "", err
	}

	return strings.TrimSpace(string(out)), nil
}

func Summary() string {
	var summary strings.Builder

	cmd := fig_proto.Command{
		Command: &fig_proto.Command_Diagnostics{},
	}

	resp, err := fig_ipc.SendRecvCommand(&cmd)
	if err != nil {
		summary.WriteString(fmt.Sprintf("Error: %s\n", err.Error()))
	}

	figVersion, _ := GetFigVersion()
	figBuild, _ := GetFigBuild()

	//  \(Diagnostic.distribution) \(Defaults.beta ? "[Beta] " : "")\(Defaults.debugAutocomplete ? "[Debug] " : "")\(Defaults.developerModeEnabled ? "[Dev] " : "")[\(KeyboardLayout.shared.currentLayoutName() ?? "?")] \(Diagnostic.isRunningOnReadOnlyVolume ? "TRANSLOCATED!!!" : "")
	summary.WriteString("Fig Version: ")
	summary.WriteString(figVersion)
	summary.WriteString(" ")
	summary.WriteString(figBuild)
	summary.WriteString("\n")

	// User shell: \(Diagnostic.userShell)
	userShell, _ := DsclRead("UserShell")
	summary.WriteString(userShell)
	summary.WriteString("\n")

	//  Bundle path: \(Diagnostic.pathToBundle)
	summary.WriteString("Bundle path: ")
	summary.WriteString(resp.GetDiagnostics().GetPathToBundle())
	summary.WriteString("\n")

	//  Autocomplete: \(Defaults.useAutocomplete)
	autocomplete, _ := ReadPlist("useAutocomplete")
	summary.WriteString("Autocomplete: ")
	summary.WriteString(autocomplete)
	summary.WriteString("\n")

	//  Settings.json: \(Diagnostic.settingsExistAndHaveValidFormat)
	_, err = settings.Load()
	if err != nil {
		summary.WriteString("Settings.json: false\n")
	} else {
		summary.WriteString("Settings.json: true\n")
	}

	//  CLI installed: \(Diagnostic.installedCLI)
	summary.WriteString("CLI installed: true\n")

	//  CLI tool path: \(Diagnostic.pathOfCLI ?? "<none>")
	executable, err := os.Executable()
	if err != nil {
		summary.WriteString("CLI tool path: <none>\n")
	} else {
		summary.WriteString(fmt.Sprintf("CLI tool path: %s\n", executable))
	}

	//  Accessibility: \(Accessibility.enabled)
	summary.WriteString("Accessibility: ")
	summary.WriteString(resp.GetDiagnostics().GetAccessibility())
	summary.WriteString("\n")

	//  Number of specs: \(Diagnostic.numberOfCompletionSpecs)
	specCount, _ := specs.GetSpecsCount()
	summary.WriteString(fmt.Sprintf("Number of specs: %d\n", specCount))

	//  SSH Integration: \(Defaults.SSHIntegrationEnabled)
	summary.WriteString("SSH Integration: false\n")

	//  Tmux Integration: \(TmuxIntegration.isInstalled)
	summary.WriteString("Tmux Integration: false\n")

	//  Keybindings path: \(Diagnostic.keybindingsPath ?? "<none>")
	summary.WriteString("Keybindings path: ")
	summary.WriteString(resp.GetDiagnostics().GetKeypath())
	summary.WriteString("\n")

	//  iTerm Integration: \(iTermIntegration.default.isInstalled) \(iTermIntegration.default.isConnectedToAPI ? "[Authenticated]": "")
	res, _ := fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationIterm)
	summary.WriteString(fmt.Sprintf("iTerm Integration: %s\n", res))

	//  Hyper Integration: \(HyperIntegration.default.isInstalled)
	res, _ = fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationHyper)
	summary.WriteString(fmt.Sprintf("Hyper Integration: %s\n", res))

	//  VSCode Integration: \(VSCodeIntegration.default.isInstalled)
	res, _ = fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationVSCode)
	summary.WriteString(fmt.Sprintf("VSCode Integration: %s\n", res))

	//  Docker Integration: \(DockerEventStream.shared.socket.isConnected)
	summary.WriteString("Docker Integration: ")
	summary.WriteString(resp.GetDiagnostics().GetDocker())
	summary.WriteString("\n")

	//  Symlinked dotfiles: \(DD)
	summary.WriteString("Symlinked dotfiles: ")
	summary.WriteString(resp.GetDiagnostics().GetSymlinked())
	summary.WriteString("\n")

	//  Only insert on tab: \(Defaults.onlyInsertOnTab)
	summary.WriteString("Only insert on tab: ")
	summary.WriteString(resp.GetDiagnostics().GetOnlytab())
	summary.WriteString("\n")

	//  Installation Script: \(Diagnostic.installationScriptRan)
	summary.WriteString("Installation Script: ")
	summary.WriteString(resp.GetDiagnostics().GetInstallscript())
	summary.WriteString("\n")

	//  PseudoTerminal Path: \(Diagnostic.pseudoTerminalPath ?? "<generated dynamically>")
	summary.WriteString("PseudoTerminal Path: ")
	summary.WriteString(resp.GetDiagnostics().GetPsudopath())
	summary.WriteString("\n")

	//  SecureKeyboardInput: \(Diagnostic.secureKeyboardInput)
	summary.WriteString("SecureKeyboardInput: ")
	summary.WriteString(resp.GetDiagnostics().GetSecurekeyboard())
	summary.WriteString("\n")

	//  SecureKeyboardProcess: \(Diagnostic.blockingProcess ?? "<none>")
	summary.WriteString("SecureKeyboardProcess: ")
	summary.WriteString(resp.GetDiagnostics().GetSecurekeyboardPath())
	summary.WriteString("\n")

	//  Current active process: \(Diagnostic.processForTopmostWindow) (\(Diagnostic.processIdForTopmostWindow)) - \(Diagnostic.ttyDescriptorForTopmostWindow)
	summary.WriteString("Current active process: ")
	summary.WriteString(resp.GetDiagnostics().GetCurrentProcess())
	summary.WriteString("\n")

	//  Current working directory: \(Diagnostic.workingDirectoryForTopmostWindow)
	wd, _ := os.Getwd()
	summary.WriteString(fmt.Sprintf("Current working directory: %s\n", wd))

	//  Current window identifier: \(Diagnostic.descriptionOfTopmostWindow)
	summary.WriteString("Current window identifier: ")
	summary.WriteString(resp.GetDiagnostics().GetCurrentWindowIdentifier())
	summary.WriteString("\n")

	// Path
	summary.WriteString("Path: ")
	summary.WriteString(os.Getenv("PATH"))

	return summary.String()
}
