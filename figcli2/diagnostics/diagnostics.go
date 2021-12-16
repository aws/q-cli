package diagnostics

import (
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fig-cli/logging"
	"fig-cli/settings"
	"fig-cli/specs"
	"fmt"
	"os"
	"os/exec"
	"os/user"
	"regexp"
	"strconv"
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

	return string(version[1]) + " " + string(build[1]), nil
}

func ReadPlist(field string) (string, error) {
	plistData, err := os.ReadFile("/Applications/Fig.app/Contents/Info.plist")
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

func GetTty() (string, error) {
	ttyExec := exec.Command("tty")
	ttyExec.Stdin = os.Stdin

	out, err := ttyExec.Output()
	if err != nil {
		return "", err
	}

	return strings.TrimSpace(string(out)), nil
}

type AppInfo string

func GetAppInfo() (AppInfo, error) {
	lsappinfoOutput, err := exec.Command("lsappinfo", "info", "-app", "com.mschrage.fig").Output()
	if err != nil {
		return AppInfo(""), err
	}

	lsappinfoTrimmed := strings.TrimSpace(string(lsappinfoOutput))
	if lsappinfoTrimmed == "" {
		return AppInfo(""), fmt.Errorf("could not determine app info")
	}

	return AppInfo(lsappinfoTrimmed), nil
}

func (a AppInfo) IsRunning() bool {
	return len(a) > 0
}

func IsFigRunning() bool {
	appInfo, err := GetAppInfo()
	if err != nil {
		return false
	}

	return appInfo.IsRunning()
}

func (a AppInfo) BundlePath() (string, error) {
	re := regexp.MustCompile(`bundle path=\"(\S+)\"`)

	matches := re.FindStringSubmatch(string(a))
	if len(matches) == 0 {
		return "", fmt.Errorf("could not determine bundle path")
	}

	return matches[1], nil
}

func (a AppInfo) BuildVersion() (string, error) {
	re := regexp.MustCompile(`Version=\"(\S+)\"`)

	matches := re.FindStringSubmatch(string(a))
	if len(matches) == 0 {
		return "", fmt.Errorf("could not determine build version")
	}

	return matches[1], nil
}

func (a AppInfo) Pid() (int, error) {
	re := regexp.MustCompile(`pid = (\S+)`)

	matches := re.FindStringSubmatch(string(a))
	if len(matches) == 0 {
		return 0, fmt.Errorf("could not determine pid")
	}

	return strconv.Atoi(matches[1])
}

func FigEnvs() []string {
	var fig_envs []string
	env := os.Environ()
	for _, e := range env {
		if strings.HasPrefix(e, "FIG_") {
			fig_envs = append(fig_envs, e)
		}
	}

	return fig_envs
}

type HardwareDiagnostics string

func GetHardwareDiagnostics() (HardwareDiagnostics, error) {
	systemProfilerOutput, err := exec.Command("system_profiler", "SPHardwareDataType").Output()
	if err != nil {
		return "", err
	}

	systemProfilerTrimmed := strings.TrimSpace(string(systemProfilerOutput))
	if systemProfilerTrimmed == "" {
		return "", fmt.Errorf("could not determine hardware diagnostics")
	}

	return HardwareDiagnostics(systemProfilerTrimmed), nil
}

func (h HardwareDiagnostics) Memory() string {
	re := regexp.MustCompile(`Memory: (.+)`)

	matches := re.FindStringSubmatch(string(h))
	if len(matches) == 0 {
		return ""
	}

	return matches[1]
}

func (h HardwareDiagnostics) Model() string {
	re := regexp.MustCompile(`Model Name: (.+)`)

	matches := re.FindStringSubmatch(string(h))
	if len(matches) == 0 {
		return ""
	}

	return matches[1]
}

func (h HardwareDiagnostics) ModelIdentifier() string {
	re := regexp.MustCompile(`Model Identifier: (.+)`)

	matches := re.FindStringSubmatch(string(h))
	if len(matches) == 0 {
		return ""
	}

	return matches[1]
}

func (h HardwareDiagnostics) Chip() string {
	re := regexp.MustCompile(`Chip: (.+)`)

	matches := re.FindStringSubmatch(string(h))
	if len(matches) == 0 {
		return ""
	}

	return matches[1]
}

func (h HardwareDiagnostics) Cores() string {
	re := regexp.MustCompile(`Total Number of Cores: (.+)`)

	matches := re.FindStringSubmatch(string(h))
	if len(matches) == 0 {
		return ""
	}

	return matches[1]
}

func Summary() string {
	var summary strings.Builder

	cmd := fig_proto.Command{
		Command: &fig_proto.Command_Diagnostics{},
	}

	resp, err := fig_ipc.SendRecvCommand(&cmd)
	if err != nil {
		summary.WriteString("\nIt looks like Fig is not running.\nTry running: fig restart\n\n")
		logging.Log("diagnostics", err.Error())
		return summary.String()
	}

	//  \(Diagnostic.distribution) \(Defaults.beta ? "[Beta] " : "")\(Defaults.debugAutocomplete ? "[Debug] " : "")\(Defaults.developerModeEnabled ? "[Dev] " : "")[\(KeyboardLayout.shared.currentLayoutName() ?? "?")] \(Diagnostic.isRunningOnReadOnlyVolume ? "TRANSLOCATED!!!" : "")
	summary.WriteString("Fig Version: ")
	summary.WriteString(resp.GetDiagnostics().GetDistribution())
	summary.WriteString(" ")

	if resp.GetDiagnostics().GetBeta() {
		summary.WriteString("[Beta] ")
	}

	if resp.GetDiagnostics().GetDebugAutocomplete() {
		summary.WriteString("[Debug] ")
	}

	if resp.GetDiagnostics().GetDeveloperModeEnabled() {
		summary.WriteString("[Dev] ")
	}

	layoutName := resp.GetDiagnostics().GetCurrentLayoutName()
	if layoutName == "" {
		layoutName = "?"
	}
	summary.WriteString(fmt.Sprintf("[%v] ", layoutName))

	if resp.GetDiagnostics().GetIsRunningOnReadOnlyVolume() {
		summary.WriteString("TRANSLOCATED!!!")
	}

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
	summary.WriteString("Autocomplete: ")
	if resp.GetDiagnostics().GetAutocomplete() {
		summary.WriteString("true")
	} else {
		summary.WriteString("false")
	}
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
	summary.WriteString(resp.GetDiagnostics().GetPsudoterminalPath())
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

	// Fig envs
	fig_envs := FigEnvs()
	summary.WriteString("\nFig environment variables:\n")
	summary.WriteString("  - TERM_SESSION_ID=")
	summary.WriteString(os.Getenv("TERM_SESSION_ID"))
	for _, env := range fig_envs {
		summary.WriteString("\n")
		summary.WriteString("  - ")
		summary.WriteString(env)
	}

	// MacOS Version
	version, _ := GetMacOsVersion()
	summary.WriteString("\nMacOS Version: ")
	summary.WriteString(version)
	summary.WriteString("\n")

	// Hardware
	hardwareDiagnostics, _ := GetHardwareDiagnostics()
	summary.WriteString("Hardware:\n")
	summary.WriteString("  - Model Name: ")
	summary.WriteString(hardwareDiagnostics.Model())
	summary.WriteString("\n")
	summary.WriteString("  - Model Identifier: ")
	summary.WriteString(hardwareDiagnostics.ModelIdentifier())
	summary.WriteString("\n")
	summary.WriteString("  - Chip: ")
	summary.WriteString(hardwareDiagnostics.Chip())
	summary.WriteString("\n")
	summary.WriteString("  - Cores: ")
	summary.WriteString(hardwareDiagnostics.Cores())
	summary.WriteString("\n")
	summary.WriteString("  - Memory: ")
	summary.WriteString(hardwareDiagnostics.Memory())

	return summary.String()
}
