package doctor

import (
	"bufio"
	"bytes"
	"errors"
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fmt"
	"net"
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
	var verbose bool

	cmd := &cobra.Command{
		Use:   "doctor",
		Short: "Check Fig is properly configured",
		Long:  "Runs a series of checks to ensure Fig is properly configured",
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
				if verbose {
					fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's make sure Fig is running...\n"))
				}

				// Check if file ~/.fig/bin/fig exists
				if _, err := os.ReadFile(fmt.Sprintf("%s/.fig/bin/fig", user.HomeDir)); err != nil {
					fmt.Println("❌ Fig bin does not exist")
				} else {
					if verbose {
						fmt.Println("✅ Fig bin exists")
					}
				}

				// Check if fig is in PATH
				path := os.Getenv("PATH")
				if !strings.Contains(path, ".fig/bin") {
					fmt.Println("❌ Fig not in PATH")
				} else {
					if verbose {
						fmt.Println("✅ Fig in PATH")
					}
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
					if verbose {
						fmt.Println("✅ Fig is running")
					}
				} else {
					if verbose {
						fmt.Println("❌ Fig is not running")
					}
					return
				}

				// Check if $TMPDIR/fig.socket exists
				fig_socket_path := filepath.Join(os.Getenv("TMPDIR"), "fig.socket")
				if _, err := os.Stat(fig_socket_path); errors.Is(err, os.ErrNotExist) {
					fmt.Println("❌ Fig socket does not exist at " + fig_socket_path)
				} else {
					if verbose {
						fmt.Println("✅ Fig socket exists at " + fig_socket_path)
					}
				}

				//Check if /tmp/figterm-$TERM_SESSION_ID.socket exists and attempt to write to it.
				figterm_socket_path := fmt.Sprintf("/tmp/figterm-%s.socket", os.Getenv("TERM_SESSION_ID"))
				if _, err := os.Stat(figterm_socket_path); errors.Is(err, os.ErrNotExist) {
					fmt.Println("❌ Figterm socket does not exist at " + figterm_socket_path)
				} else {
					// Attempt to write to the socket
					conn, err := net.Dial("unix", figterm_socket_path)
					if err != nil {
						fmt.Println("❌ Figterm socket exists but is not connectable")
						fmt.Printf("   %v\n", err.Error())
					} else {
						go func() {
							defer conn.Close()
							time.Sleep(time.Millisecond * 10)
							conn.Write([]byte("Testing Figterm...\n"))
						}()

						reader := bufio.NewReader(os.Stdin)
						val, _ := reader.ReadString('\n')

						if strings.Contains(val, "Testing Figterm...") {
							if verbose {
								fmt.Println("✅ Figterm socket exists and is writable")
							}
						} else {
							fmt.Println("❌ Figterm socket exists but is not writable")
						}
					}
				}

				if verbose {
					fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check your dotfiles...\n"))
				}

				for _, fileName := range []string{".profile", ".zprofile", ".bash_profile", ".bashrc", ".zshrc"} {
					for {
						// Read file if it exists
						fileData, err := os.ReadFile(filepath.Join(user.HomeDir, fileName))

						if err == nil {
							// Strip comments lines out of file
							r := regexp.MustCompile(`\s*#.*`)
							strippedData := r.ReplaceAll(fileData, []byte(""))

							// Only lines that contain 'PATH|source'
							r = regexp.MustCompile(`.*(PATH|source).*`)
							lines := r.FindAll(strippedData, -1)

							boldStyle := lipgloss.NewStyle().Bold(true)
							codeStyle := lipgloss.NewStyle().Foreground(lipgloss.Color("5"))

							if len(lines) == 0 ||
								!bytes.Equal(lines[0], []byte(`[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh`)) {

								fmt.Printf("\n%v\n", boldStyle.Render(fmt.Sprintf("\n❌ Shell integration must be sourced first in ~/%s", fileName)))
								fmt.Printf("   In order for autocomplete to work correctly, Fig's shell integration must be sourced first.\n\n")
								fmt.Printf("   Please add the following line to the top of your ~/%s file:\n", fileName)
								fmt.Printf("   %v\n", codeStyle.Render("[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh"))
								fmt.Printf("\n")
								fmt.Printf("   Once you have updated `~/%s`, press `enter` to continue: ", fileName)

								reader := bufio.NewReader(os.Stdin)
								reader.ReadString('\n')
								continue
							} else {
								if verbose {
									fmt.Printf("✅ ~/.fig/shell/pre.sh sourced first in ~/%s\n", fileName)
								}
							}

							if len(lines) >= 1 &&
								!bytes.Equal(lines[len(lines)-1], []byte(`[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh`)) {

								fmt.Printf("\n%v\n", boldStyle.Render(fmt.Sprintf("\n❌ Fig shell integration must be sourced last in ~/%s", fileName)))
								fmt.Printf("   In order for autocomplete to work correctly, Fig's shell integration must be sourced last.\n\n")
								fmt.Printf("   Please add the following line to the bottom of your ~/%s file:\n", fileName)
								fmt.Printf("   %v\n", codeStyle.Render("[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh"))
								fmt.Printf("\n")
								fmt.Printf("   Once you have updated `~/%s`, press `enter` to continue: ", fileName)

								reader := bufio.NewReader(os.Stdin)
								reader.ReadString('\n')
								continue
							} else {
								if verbose {
									fmt.Println("✅ ~/.fig/fig.sh sourced last in ~/fig.sh")
								}
							}

							break
						} else {
							if verbose {
								fmt.Printf("🟡 ~/%s does not exist\n", fileName)
							}
							break
						}
					}
				}

				if verbose {
					fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check if your system is compatible...\n"))
				}

				// Check if darwin
				if runtime.GOOS == "darwin" {
					if verbose {
						fmt.Println("✅ Running macOS")
					}
				} else {
					fmt.Println("❌ Running " + runtime.GOOS)
					return
				}

				macosVersion, err := exec.Command("sw_vers", "-productVersion").Output()
				if err != nil {
					if verbose {
						fmt.Println("❌ Could not get macOS version")
					}
					return
				}

				macosVersionSplit := strings.Split(string(macosVersion), ".")
				majorVersion, _ := strconv.Atoi(macosVersionSplit[0])
				minorVersion, _ := strconv.Atoi(macosVersionSplit[1])

				if majorVersion >= 11 {
					if verbose {
						fmt.Println("✅ macOS version is 11.x or higher")
					}
				} else {
					if majorVersion == 10 && minorVersion >= 14 {
						if verbose {
							fmt.Println("✅ macOS version is 10.14 or higher")
						}
					} else {
						fmt.Println("❌ macOS version lower than 10.14 is incompatible with Fig")
					}
				}

				if verbose {
					fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check what ") +
						lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Bold(true).Italic(true).Render("fig diagnostic") +
						lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render(" says...\n"))
				}

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
					if verbose {
						fmt.Println("✅ Installation script")
					}
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
					if verbose {
						fmt.Println("✅ Shell " + lipgloss.NewStyle().Bold(true).Render(currentShell) + " is compatible")
					}
				} else if !userShellCompatible && !currentShellCompatible {
					fmt.Println()
					fmt.Println("❌ Shell " + lipgloss.NewStyle().Bold(true).Render(currentShell) + " is incompatible")
				} else {
					fmt.Println()
					if userShellCompatible {
						if verbose {
							fmt.Println("✅ Default shell " + lipgloss.NewStyle().Bold(true).Render(userShell) + " is compatible")
						}
					}

					if currentShellCompatible {
						if verbose {
							fmt.Println("✅ Current shell " + lipgloss.NewStyle().Bold(true).Render(currentShell) + " is compatible")
						}
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
					if verbose {
						fmt.Println("✅ Fig is installed in " + lipgloss.NewStyle().Bold(true).Render(bundlePath))
					}
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
					if verbose {
						fmt.Println("✅ Autocomplete is enabled")
					}
				} else {
					fmt.Println()
					fmt.Println("❌ Autocomplete is disabled")
					fmt.Println("   To fix run: " + lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("fig settings autocomplete.disable false"))
					fmt.Println()
				}

				// CLI Path
				executable, err := os.Executable()
				if err != nil {
					if verbose {
						fmt.Println("❌ Could not get Fig executable path")
					}
				} else {
					if executable == filepath.Join(user.HomeDir, ".fig/bin/fig") ||
						executable == "/usr/local/bin/.fig/bin/fig" ||
						executable == "/usr/local/bin/fig" {
						if verbose {
							fmt.Println("✅ CLI tool path")
						}
					} else {
						fmt.Println()
						fmt.Println("❌ CLI tool path")
						fmt.Printf("   The Fig CLI must be in %s/.fig/bin/fig\n", user.HomeDir)
						fmt.Println()
					}
				}

				// Accessibility
				if diagnosticsResp.GetDiagnostics().GetAccessibility() == "true" {
					if verbose {
						fmt.Println("✅ Accessibility is enabled")
					}
				} else {
					fmt.Println("❌ Accessibility is disabled")
					Fix("fig debug prompt-accessibility")
					continue
				}

				// Path
				if diagnosticsResp.GetDiagnostics().GetPsudoterminalPath() == os.Getenv("PATH") {
					if verbose {
						fmt.Println("✅ PATH and PseudoTerminal PATH match")
					}
				} else {
					fmt.Println("❌ PATH and PseudoTerminal PATH do not match")
					Fix("fig app set-path")
					continue
				}

				// SecureKeyboardProcess
				if diagnosticsResp.GetDiagnostics().GetSecurekeyboard() == "false" {
					if verbose {
						fmt.Println("✅ Secure keyboard input")
					}
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
				if verbose {
					fmt.Println(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("\nLet's check your integration statuses...\n"))
				}

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
									if verbose {
										fmt.Println("✅ iTerm integration is enabled")
									}
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
							if verbose {
								fmt.Println("✅ iTerm Bash Integration is up to date.")
							}
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
						if verbose {
							fmt.Println("✅ Hyper integration is enabled")
						}
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
						if verbose {
							fmt.Println("✅ VSCode integration is enabled")
						}
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

	cmd.Flags().BoolVarP(&verbose, "verbose", "v", false, "Show verbose output")

	return cmd
}
