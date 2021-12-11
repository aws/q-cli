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
	"time"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func Fix(cmd string) {
	user, err := user.Current()
	if err != nil {
		fmt.Println("Could not determine current user")
		return
	}

	// Read fixes file
	fixFile := filepath.Join(user.HomeDir, ".fig", "fig_fixes")
	fixFileData, err := os.ReadFile(fixFile)
	if err != nil && !os.IsNotExist(err) {
		fmt.Println("Could not read fixes file")
		os.Exit(1)
	}

	if err == nil {
		// Remove file
		os.Remove(fixFile)

		// Check if fix cmd is in fixes file
		if bytes.Contains(fixFileData, []byte(cmd)) {
			fmt.Printf("\nLooks like we've already tried this fix before and it's not working.\n")
			ContactSupport()
			os.Exit(1)
		}
	}

	// Run fix cmd
	fmt.Printf("\nI can fix this!\n")

	// Append to fixes file
	f, err := os.OpenFile(fixFile, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err == nil {
		defer f.Close()
		f.Write([]byte(cmd + "\n"))
	}

	fmt.Printf("Running > " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render(cmd) + "\n\n")
	executeCmd := exec.Command("sh", "-c", cmd)
	executeCmd.Stdout = os.Stdout
	executeCmd.Stderr = os.Stderr
	executeCmd.Stdin = os.Stdin
	err = executeCmd.Run()

	if err != nil {
		fmt.Println("Could not fix this!")
		ContactSupport()
		return
	}

	fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("2")).Render("\nFix applied!"))
	fmt.Printf("Rerunning " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig doctor") + " to see if the problem is resolved")
	// Sleep 5 seconds
	time.Sleep(time.Second)
	fmt.Printf(".")
	time.Sleep(time.Second)
	fmt.Printf(".")
	time.Sleep(time.Second)
	fmt.Printf(".")
	time.Sleep(time.Second)
	fmt.Printf(".")
	time.Sleep(time.Second)
	fmt.Printf(".\n\n")

}

func ContactSupport() {
	fmt.Printf("\nRun " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig issue") + " to let us know about this error!\n")
	fmt.Printf("Or, email us at " + lipgloss.NewStyle().Underline(true).Foreground(lipgloss.Color("6")).Render("hello@fig.io") + "!\n\n")
}

func IsInstalled(application string) bool {
	listInsatlledApps, err := exec.Command("mdfind", "kMDItemKind == 'Application'").Output()
	if err != nil {
		return false
	}

	installedApps := strings.Split(string(listInsatlledApps), "\n")
	for _, app := range installedApps {
		if strings.Contains(app, application) {
			return true
		}
	}

	return false
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
			// Get user
			user, err := user.Current()
			if err != nil {
				fmt.Printf("\n%v\n", err)
				ContactSupport()
				return
			}

			// Remove fix file
			fixFile := filepath.Join(user.HomeDir, ".fig", "fig_fixes")
			os.Remove(fixFile)

			for {
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's make sure Fig is running...\n"))

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
					Fix("fig launch")
					continue
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
							fmt.Println(style.Render("   If you don't see the FIG ENV VARs in ~/" + fileName + ", run 'fig app install' to add them. Open a new terminal then rerun the doctor."))
						} else {
							fmt.Printf("✅ Fig ENV variables are in ~/%s\n", fileName)
						}

					}
				}

				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check if your system is compatible...\n"))

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

				if majorVersion >= 11 {
					fmt.Println("✅ macOS version is 11.x or higher")
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
					fmt.Println("   Try restarting Fig by running:" + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render(" fig restart"))
					fmt.Println("\nIf you are still running into this error:")
					ContactSupport()
					return
				}

				// Installation Script
				if diagnosticsResp.GetDiagnostics().GetInstallscript() == "true" {
					fmt.Println("✅ Installation script")
				} else {
					fmt.Println("❌ Installation script")
					Fix("~/.fig/tools/install_and_upgrade.sh")
					continue
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
						executable == "/usr/local/bin/.fig/bin/fig" ||
						executable == "/usr/local/bin/fig" {
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
					Fix("fig debug prompt-accessibility")
					continue
				}

				// Path
				if diagnosticsResp.GetDiagnostics().GetPsudoterminalPath() == os.Getenv("PATH") {
					fmt.Println("✅ PATH and PseudoTerminal PATH match")
				} else {
					fmt.Println("❌ PATH and PseudoTerminal PATH do not match")
					Fix("fig app set-path")
					continue
				}

				// SecureKeyboardProcess
				if diagnosticsResp.GetDiagnostics().GetSecurekeyboard() == "false" {
					fmt.Println("✅ Secure keyboard input")
				} else {
					if IsInstalled("Bitwarden.app") {
						// Check bitwarden version
						bitwardenVersion, err := exec.Command("mdls", "-name", "kMDItemVersion", "/Applications/Bitwarden.app").Output()
						if err != nil {
							fmt.Println()
							fmt.Println("❌ Could not get Bitwarden version")
							fmt.Println("❌ Secure keyboard input")
							fmt.Println("   Secure keyboard input is on")
							fmt.Println("   Secure keyboard process is", diagnosticsResp.GetDiagnostics().GetSecurekeyboardPath())
							fmt.Println()

						} else {
							versionRegex := regexp.MustCompile(`(\d+)\.(\d+)`)
							versionMatch := versionRegex.FindStringSubmatch(string(bitwardenVersion))
							if len(versionMatch) == 3 {
								major, _ := strconv.Atoi(versionMatch[1])
								minor, _ := strconv.Atoi(versionMatch[2])
								if major <= 1 && minor <= 27 {
									fmt.Println()
									fmt.Println("❌ Secure keyboard input is")
									fmt.Println("   Bitwarden may be enabling secure keyboard entry even when not focused.")
									fmt.Println("   This was fixed in version 1.28.0. See https://github.com/bitwarden/desktop/issues/991 for details.")
									fmt.Println("   To fix: upgrade Bitwarden to the latest version")
									fmt.Println()
								} else {
									fmt.Println()
									fmt.Println("❌ Secure keyboard input")
									fmt.Println("   Secure keyboard input is on")
									fmt.Println("   Secure keyboard process is", diagnosticsResp.GetDiagnostics().GetSecurekeyboardPath())
									fmt.Println()
								}
							}
						}
					} else {
						fmt.Println()
						fmt.Println("❌ Secure keyboard input")
						fmt.Println("   Secure keyboard input is on")
						fmt.Println("   Secure keyboard process is", diagnosticsResp.GetDiagnostics().GetSecurekeyboardPath())
						fmt.Println()
					}
				}

				// Integrations
				fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check your integration statuses...\n"))

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
						// Check iTerm version
						itermVersion, err := exec.Command("mdls", "-name", "kMDItemVersion", "/Applications/iTerm.app").Output()
						if err != nil {
							fmt.Println("❌ Could not get iTerm version")
						} else {
							versionRegex := regexp.MustCompile(`(\d+)\.(\d+)\.(\d+)`)
							versionMatch := versionRegex.FindStringSubmatch(string(itermVersion))
							if len(versionMatch) == 4 {
								itermVersionMajor, _ := strconv.Atoi(versionMatch[1])
								itermVersionMinor, _ := strconv.Atoi(versionMatch[2])
								if itermVersionMajor >= 3 && itermVersionMinor >= 4 {
									fmt.Println("✅ iTerm integration is enabled")
								} else {
									fmt.Println("❌ iTerm integration fail")
									fmt.Println("   Your iTerm version is incompatible with Fig. Please update iTerm to latest version")
								}
							}
						}
					} else {
						// Check if iTerm is installed
						if IsInstalled("iTerm.app") {
							fmt.Println()
							fmt.Println("❌ iTerm integration fail")

							// Check if API is enabled
							apiEnabled, err := exec.Command("defaults", "read", "com.googlecode.iterm2", "EnableAPIServer").Output()
							if err != nil {
								fmt.Println("   Could not get iTerm API status")
							} else if string(apiEnabled) == "0\n" {
								fmt.Println("   The iTerm API server is not enabled.")
							}

							// Check that fig-iterm-integration.scpt exists in ~/Library/Application\ Support/iTerm2/Scripts/AutoLaunch/
							itermIntegrationPath := filepath.Join(user.HomeDir, "Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt")
							if _, err := os.Stat(itermIntegrationPath); os.IsNotExist(err) {
								fmt.Println("   fig-iterm-integration.scpt is missing.")
							}
						}
					}
				}

				// iTerm Shell Integration Pre-exec Version
				// Read fixes file regexp.FindString( )
				iterm2ShellIntegrationFile := filepath.Join(user.HomeDir, ".iterm2_shell_integration.bash")
				iterm2ShellIntegration, err := os.ReadFile(iterm2ShellIntegrationFile)
				if err != nil && !os.IsNotExist(err) {
					fmt.Println("❌ Could not read .iterm2_shell_integration.bash file")
				} else if err == nil {
					preexecVersionRegex := regexp.MustCompile(`V(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)`)
					version := preexecVersionRegex.FindStringSubmatch(string(iterm2ShellIntegration))
					if len(version) < 3 {
						fmt.Println("🟡 You have iTerm's Bash Integration installed, but we could not check the version in ~/.iterm2_shell_integration.bash. Integration may be out of date. You can try updating in iTerm's menu by selecting \"Install Shell Integration\"")
					} else {
						major, _ := strconv.Atoi(version[1])
						minor, _ := strconv.Atoi(version[2])
						if major > 0 || minor > 3 {
							fmt.Println("✅ iTerm Bash Integration is up to date.")
						} else {
							fmt.Println("❌ iTerm Bash Integration is out of date. Please update in iTerm's menu by selecting \"Install Shell Integration\".")
						}
					}
				}

				// Hyper Integration
				hyperIntegration, err := fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationHyper)
				if err != nil {
					fmt.Println("❌ Could not verify Hyper integration")
				} else {
					if hyperIntegration == "installed!" {
						fmt.Println("✅ Hyper integration is enabled")
					} else {
						// Check if Hyper is installed
						if IsInstalled("Hyper.app") {
							fmt.Println()
							fmt.Println("❌ Hyper integration fail")

							// Check ~/.hyper_plugins/local/fig-hyper-integration/index.js exists
							hyperIntegrationPath := filepath.Join(user.HomeDir, ".hyper_plugins/local/fig-hyper-integration/index.js")
							if _, err := os.Stat(hyperIntegrationPath); os.IsNotExist(err) {
								fmt.Println("   fig-hyper-integration plugin is missing!.")
							}

							// Check if plugin is enabled in ~/.hyper.js
							hyperConfigPath := filepath.Join(user.HomeDir, ".hyper.js")
							if _, err := os.Stat(hyperConfigPath); os.IsNotExist(err) {
								fmt.Println("   ~/.hyper.js is missing.")
							} else {
								hyperConfig, err := os.ReadFile(hyperConfigPath)
								if err != nil {
									fmt.Println("   Could not read ~/.hyper.js")
								} else {
									if !strings.Contains(string(hyperConfig), "fig-hyper-integration") {
										fmt.Println("   fig-hyper-integration plugin needs to be added to localPlugins!")
									}
								}
							}
						}
					}
				}

				// VSCode Integration
				vscodeIntegration, err := fig_ipc.IntegrationVerifyInstall(fig_ipc.IntegrationVSCode)
				if err != nil {
					fmt.Println("❌ Could not verify VSCode integration")
				} else {
					if vscodeIntegration == "installed!" {
						fmt.Println("✅ VSCode integration is enabled")
					} else {
						if IsInstalled("Visual Studio Code.app") {
							fmt.Println("❌ VSCode integration fail")

							// Check if withfig.fig exists
							files, err := filepath.Glob(filepath.Join(user.HomeDir, ".vscode", "extensions", "withfig.fig-*"))
							if err != nil || len(files) == 0 {
								fmt.Println("   VSCode extension is missing!")
							}
						}
					}
				}

				// Debug Mode check
				debugMode, err := fig_ipc.GetDebugModeCommand()
				if err != nil {
					fmt.Println("❌ Could not get debug mode")
				} else {
					if debugMode == "on" {
						fmt.Println()
						fmt.Println("🟡 Debug mode is enabled")
						fmt.Println("   Disable by running: " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig debug debug-mode off"))
						fmt.Println()
					}
				}

				if diagnosticsResp.GetDiagnostics().GetSymlinked() == "true" {
					fmt.Println("FYI, looks like your dotfiles are symlinked.")
					fmt.Println("If you need to make modifications, make sure they're made in the right place.")
				}

				fmt.Println()
				fmt.Println("Fig still not working?")
				fmt.Println("Run " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig issue") + " to let us know!")
				fmt.Println("Or, email us at " + lipgloss.NewStyle().Underline(true).Foreground(lipgloss.Color("6")).Render("hello@fig.io") + "!")
				fmt.Println()

				break
			}
		},
	}

	return cmd
}
