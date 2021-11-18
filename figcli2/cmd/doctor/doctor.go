package doctor

import (
	"bytes"
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fmt"
	"os"
	"os/exec"
	"os/user"
	"path/filepath"
	"regexp"
	"runtime"
	"strconv"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func ContactSupport() {
	fmt.Printf("\nRun " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig issue") + " to let us know about this error!\n")
	fmt.Printf("Or, email us at " + lipgloss.NewStyle().Foreground(lipgloss.Color("6")).Render("hello@fig.io") + "!\n\n")
}

func NewCmdDoctor() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "doctor",
		Short: "Check Fig is properly configured",
		Long:  "Runs a series of checks to ensure Fig is properly configured",
		Annotations: map[string]string{
			"figcli.command.categories": "Common",
		},
		Run: func(cmd *cobra.Command, args []string) {
			for {
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's make sure Fig is running...\n"))

				// Get user
				user, err := user.Current()
				if err != nil {
					ContactSupport()
					fmt.Println(err)
					return
				}

				// Check if file ~/.fig/bin/fig exists
				if _, err := os.ReadFile(fmt.Sprintf("%s/.fig/bin/fig", user.HomeDir)); err != nil {
					fmt.Println("❌ Fig bin does not exist")
				} else {
					fmt.Println("✅ Fig bin exists")
				}

				// Check if fig is in PATH
				path := os.Getenv("PATH")
				if !strings.Contains(path, ".fig/bin") {
					fmt.Println("❌ Fig not in PATH")
				} else {
					fmt.Println("✅ Fig in PATH")
				}

				// Check if fig is running
				appInfo, err := diagnostics.GetAppInfo()
				if err != nil {
					fmt.Println("❌ Fig is not running")
					return
				}

				running := appInfo.IsRunning()

				if running {
					fmt.Println("✅ Fig is running")
				} else {
					fmt.Println("❌ Fig is not running")

					return
				}

				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check your dotfiles...\n"))

				for _, fileName := range []string{".profile", ".zprofile", ".bash_profile", ".bashrc", ".zshrc"} {
					// Read file if it exists
					fileData, err := os.ReadFile(filepath.Join(user.HomeDir, fileName))

					if err == nil {
						// Strip comments lines out of file
						r := regexp.MustCompile(`\s*#.*`)
						fileData = r.ReplaceAll(fileData, []byte(""))

						// Only lines that contain 'PATH|source'
						r = regexp.MustCompile(`.*(PATH|source).*`)
						lines := r.FindAll(fileData, -1)

						first := lines[0]
						last := lines[len(lines)-1]

						if !bytes.Equal(first, []byte(`[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh`)) ||
							!bytes.Equal(last, []byte(`[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh`)) {
							fmt.Printf("\n🟡 Fig ENV variables not properly set in ~/%s\n", fileName)

							style := lipgloss.NewStyle().Foreground(lipgloss.Color("3"))

							fmt.Println(style.Render("   Fig ENV variables need to be at the very beginning and end of ~/" + fileName))
							fmt.Println(style.Render("   If you see the FIG ENV VARs in ~/" + fileName + ", make sure they're at the very beginning (pre) and end (post). Open a new terminal then rerun the the doctor."))
							fmt.Println(style.Render("   If you don't see the FIG ENV VARs in ~/" + fileName + ", run 'fig util:install-script' to add them. Open a new terminal then rerun the doctor."))
						} else {
							fmt.Printf("✅ Fig ENV variables are in ~/%s\n", fileName)
						}

					}
				}

				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check your if your system is compatible...\n"))

				// Check if darwin
				if runtime.GOOS == "darwin" {
					fmt.Println("✅ Running macOS")
				} else {
					fmt.Println("❌ Running " + runtime.GOOS)
					return
				}

				macosVersion, err := exec.Command("sw_vers", "-productVersion").Output()
				if err != nil {
					fmt.Println("❌ Could not get macOS version")
					return
				}

				macosVersionSplit := strings.Split(string(macosVersion), ".")
				majorVersion, _ := strconv.Atoi(macosVersionSplit[0])
				minorVersion, _ := strconv.Atoi(macosVersionSplit[1])

				if majorVersion > 10 {
					fmt.Println("✅ macOS version is 10.x or higher")
				} else {
					if majorVersion == 10 && minorVersion >= 14 {
						fmt.Println("✅ macOS version is 10.14 or higher")
					} else {
						fmt.Println("❌ macOS version lower than 10.14 is incompatible with Fig")
					}
				}

				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check what ") +
					lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Bold(true).Italic(true).Render("fig diagnostic") +
					lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render(" says...\n"))

				cmd := fig_proto.Command{
					Command: &fig_proto.Command_Diagnostics{},
				}

				diagnosticsResp, err := fig_ipc.SendRecvCommand(&cmd)
				if err != nil {
					fmt.Println("❌ Unable to get diagnostics")
					ContactSupport()
					return
				}

				// Installation Script
				if diagnosticsResp.GetDiagnostics().GetInstallscript() == "true" {
					fmt.Println("✅ Installation script")
				} else {
					fmt.Println("❌ Installation script")
				}

				// Current Shell and User Shell
				compatibleShellsRegex := regexp.MustCompile(`(bash|zsh|fish)`)

				userShell, err := diagnostics.DsclRead("UserShell")
				if err != nil {
					fmt.Println("🟡 Could not get current user shell")
				}
				userShell = strings.TrimPrefix(userShell, "UserShell: ")

				currentShell, err := fig_ipc.GetShell()
				if err != nil {
					fmt.Println("🟡 Could not get current shell")
				}

				userShellCompatible := compatibleShellsRegex.MatchString(userShell)
				currentShellCompatible := compatibleShellsRegex.MatchString(currentShell)

				if userShellCompatible && currentShellCompatible {
					fmt.Println("✅ Shell " + lipgloss.NewStyle().Bold(true).Render(currentShell) + " is compatible")
				} else if !userShellCompatible && !currentShellCompatible {
					fmt.Println()
					fmt.Println("❌ Shell " + lipgloss.NewStyle().Bold(true).Render(currentShell) + " is incompatible")
				} else {
					fmt.Println()
					if userShellCompatible {
						fmt.Println("✅ Default shell " + lipgloss.NewStyle().Bold(true).Render(userShell) + " is compatible")
					}

					if currentShellCompatible {
						fmt.Println("✅ Current shell " + lipgloss.NewStyle().Bold(true).Render(currentShell) + " is compatible")
					}

					if !userShellCompatible {
						fmt.Println("❌ Default shell " + lipgloss.NewStyle().Bold(true).Render(userShell) + " is not compatible")
					}

					if !currentShellCompatible {
						fmt.Println("❌ Current shell " + lipgloss.NewStyle().Bold(true).Render(currentShell) + " is not compatible")
					}
				}

				if !userShellCompatible || !currentShellCompatible {
					fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("3")).Render("   You are not using a supported shell."))
					fmt.Println("   Only " + "zsh" + ", " + "bash" + ", or " + "fish" + " are integrated with Fig.")
					fmt.Println()
				}

				// Bundle path
				bundlePath := diagnosticsResp.GetDiagnostics().GetPathToBundle()
				if strings.Contains(bundlePath, "/Applications/Fig.app") {
					fmt.Println("✅ Fig is installed in " + lipgloss.NewStyle().Bold(true).Render(bundlePath))
				} else if strings.Contains(bundlePath, "/Build/Products/Debug/fig.app") {
					fmt.Println("🟡 Fig is running debug build in " + lipgloss.NewStyle().Bold(true).Render(bundlePath))
				} else {
					fmt.Println()
					fmt.Println("❌ Fig is installed in " + lipgloss.NewStyle().Bold(true).Render(bundlePath))
					fmt.Println("   You need to install Fig in /Applications.")
					fmt.Println("   To fix: uninstall, then reinstall Fig.")
					fmt.Println("   Remember to drag Fig into the Applications folder.")
					fmt.Println()
				}

				// Autocomplete
				if diagnosticsResp.GetDiagnostics().GetAutocomplete() {
					fmt.Println("✅ Autocomplete is enabled")
				} else {
					fmt.Println()
					fmt.Println("❌ Autocomplete is disabled")
					fmt.Println("  To fix run: " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig settings autocomplete.disable false"))
					fmt.Println()
				}

				// CLI Path
				executable, err := os.Executable()
				if err != nil {
					fmt.Println("❌ Could not get Fig executable path")
				} else {
					if executable == filepath.Join(user.HomeDir, ".fig/bin/fig") ||
						executable == "/usr/local/bin/.fig/bin/fig" {
						fmt.Println("✅ CLI tool path")
					} else {
						fmt.Println()
						fmt.Println("❌ CLI tool path")
						fmt.Printf("   The Fig CLI must be in %s/.fig/bin/fig\n", user.HomeDir)
						fmt.Println()
					}
				}

				// Accessibility
				if diagnosticsResp.GetDiagnostics().GetAccessibility() == "true" {
					fmt.Println("✅ Accessibility is enabled")
				} else {
					fmt.Println("❌ Accessibility is disabled")
				}

				// Path
				if diagnosticsResp.GetDiagnostics().GetPsudoterminalPath() == os.Getenv("PATH") {
					fmt.Println("✅ PATH and PseudoTerminal PATH match")
				} else {
					fmt.Println()
					fmt.Println("🟡 PATH and PseudoTerminal PATH do not match")
					fmt.Println("   To fix run: " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig app set-path"))
					fmt.Println()
				}

				// SecureKeyboardProcess
				if diagnosticsResp.GetDiagnostics().GetSecurekeyboard() == "false" {
					fmt.Println("✅ Secure keyboard input")
				} else {
					fmt.Println()
					fmt.Println("❌ Secure keyboard input")
					fmt.Println()
				}

				// Integrations
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check your your integration statuses...\n"))

				// SSH Integration
				// TODO

				// Tmux Integration
				// TODO

				// iTerm Integration
				itermIntegration, err := fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationIterm)
				if err != nil {
					fmt.Println("❌ Could not verify iTerm integration")
				} else {
					if itermIntegration == "installed!" {
						fmt.Println("✅ iTerm integration is enabled")
					}
				}

				// Hyper Integration
				hyperIntegration, err := fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationHyper)
				if err != nil {
					fmt.Println("❌ Could not verify Hyper integration")
				} else {
					if hyperIntegration == "installed!" {
						fmt.Println("✅ Hyper integration is enabled")
					}
				}

				// VSCode Integration
				vscodeIntegration, err := fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationVSCode)
				if err != nil {
					fmt.Println("❌ Could not verify VSCode integration")
				} else {
					if vscodeIntegration == "installed!" {
						fmt.Println("✅ VSCode integration is enabled")
					}
				}
				
				// Debug Mode check
				

				break
			}
		},
	}

	return cmd
}
