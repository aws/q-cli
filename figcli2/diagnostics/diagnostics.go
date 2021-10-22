package diagnostics

import (
	fig_ipc "fig-cli/fig-ipc"
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

	re, err := regexp.Compile(fmt.Sprintf("<key>%s</key>\\s*<\\S>(.*)</\\S>", field))
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

// NOTE: this does not work when using `go run`, you must use `go build`
func GetShell() (string, error) {
	parentId := os.Getppid()
	execPs, err := exec.Command("ps", "-p", fmt.Sprintf("%d", parentId), "-o", "comm=").Output()
	if err != nil {
		return "unknown", err
	}

	return strings.TrimSpace(string(execPs)), nil
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

func Summary() string {
	var summary strings.Builder

	//  \(Diagnostic.distribution) \(Defaults.beta ? "[Beta] " : "")\(Defaults.debugAutocomplete ? "[Debug] " : "")\(Defaults.developerModeEnabled ? "[Dev] " : "")[\(KeyboardLayout.shared.currentLayoutName() ?? "?")] \(Diagnostic.isRunningOnReadOnlyVolume ? "TRANSLOCATED!!!" : "")
	

	// User shell: \(Diagnostic.userShell)
	userShell, _ := DsclRead("UserShell")
	summary.WriteString(userShell)
	summary.WriteString("\n")

	//  Bundle path: \(Diagnostic.pathToBundle)

	//  Autocomplete: \(Defaults.useAutocomplete)

	//  Settings.json: \(Diagnostic.settingsExistAndHaveValidFormat)
	_, err := settings.Load()
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

	//  Number of specs: \(Diagnostic.numberOfCompletionSpecs)
	specCount, _ := specs.GetSpecsCount()
	summary.WriteString(fmt.Sprintf("Number of specs: %d\n", specCount))

	//  SSH Integration: \(Defaults.SSHIntegrationEnabled)
	summary.WriteString("SSH Integration: false\n")

	//  Tmux Integration: \(TmuxIntegration.isInstalled)
	summary.WriteString("Tmux Integration: false\n")

	//  Keybindings path: \(Diagnostic.keybindingsPath ?? "<none>")

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
	//  Symlinked dotfiles: \(Diagnostic.dotfilesAreSymlinked)
	//  Only insert on tab: \(Defaults.onlyInsertOnTab)
	//  Installation Script: \(Diagnostic.installationScriptRan)
	//  PseudoTerminal Path: \(Diagnostic.pseudoTerminalPath ?? "<generated dynamically>")
	//  SecureKeyboardInput: \(Diagnostic.secureKeyboardInput)
	//  SecureKeyboardProcess: \(Diagnostic.blockingProcess ?? "<none>")
	//  Current active process: \(Diagnostic.processForTopmostWindow) (\(Diagnostic.processIdForTopmostWindow)) - \(Diagnostic.ttyDescriptorForTopmostWindow)

	//  Current working directory: \(Diagnostic.workingDirectoryForTopmostWindow)
	wd, _ := os.Getwd()
	summary.WriteString(fmt.Sprintf("Current working directory: %s\n", wd))

	//  Current window identifier: \(Diagnostic.descriptionOfTopmostWindow)

	return summary.String()
}
