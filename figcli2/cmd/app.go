package cmd

import (
	"encoding/json"
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/settings"
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"os/user"
	"regexp"
	"strings"
	"time"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func init() {
	// appCmd.AddCommand(appUpdateSpecsCmd)
	appCmd.AddCommand(appOnboardingCmd)
	appCmd.AddCommand(appThemeCmd)
	appCmd.AddCommand(appUpgradeCmd)
	appCmd.AddCommand(appSetPath)
	appCmd.AddCommand(appUninstallCmd)
	appCmd.AddCommand(appRunningCmd)

	rootCmd.AddCommand(appCmd)
}

var appCmd = &cobra.Command{
	Use:   "app",
	Short: "Manage your Fig app",
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},
}

// var appUpdateSpecsCmd = &cobra.Command{
// 	Use:   "update-specs",
// 	Short: "Update repo of completion scripts",
// 	Run: func(cmd *cobra.Command, arg []string) {
// 		fmt.Println()
// 		fmt.Println("Pulling most up-to-date completion specs...")
// 		fmt.Println("Run " + lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#FF00FF")).Render("fig docs") + " to learn how to contribute your own!")
// 		fmt.Println()

// 		usr, err := user.Current()
// 		if err != nil {
// 			fmt.Println(err)
// 			return
// 		}

// 		autocompleteVersion, err := exec.Command("defaults", "read", "com.mschrage.fig", "autocompleteVersion").Output()
// 		if err != nil {
// 			fmt.Println("Error reading autocomplete version:", err)
// 			return
// 		}

// 		autocompleteVersionStr := strings.TrimSpace(string(autocompleteVersion))

// 		build, err := exec.Command("defaults", "read", "com.mschrage.fig", "build").Output()
// 		if err != nil {
// 			fmt.Println("Error getting build number:", err)
// 			return
// 		}

// 		buildStr := strings.TrimSpace(string(build))

// 		appVersion, err := exec.Command("fig", "--version").Output()
// 		if err != nil {
// 			fmt.Println("Error getting fig version:", err)
// 			return
// 		}

// 		appVersionStr := strings.TrimSpace(string(appVersion))

// 		// Make directory if it doesn't exist at ~/.fig
// 		if _, err := os.Stat(usr.HomeDir + "/.fig/autocomplete"); os.IsNotExist(err) {
// 			os.Mkdir(usr.HomeDir+"/.fig/autocomplete", 0755)
// 		}

// 		os.Chdir(usr.HomeDir + "/.fig/autocomplete")

// 		// Download autocomplete script and pipe it to tar
// 		data, err := http.Get("https://api.fig.io/specs?version=" + autocompleteVersionStr + "&app=" + appVersionStr + "&build=" + buildStr)
// 		if err != nil {
// 			fmt.Println("Error downloading completion specs:", err)
// 			return
// 		}

// 		tar := exec.Command("tar", "-xz", "--strip-components=1", "specs")
// 		tar.Stdin = data.Body
// 		tar.Stdout = os.Stdout
// 		tar.Stderr = os.Stderr
// 		tar.Run()

// 	},
// }

var appOnboardingCmd = &cobra.Command{
	Use:   "onboarding",
	Short: "Run through onboarding process",
	Run: func(cmd *cobra.Command, arg []string) {
		sh := exec.Command("bash", "-c", "~/.fig/tools/drip/fig_onboarding.sh")
		sh.Stdout = os.Stdout
		sh.Stderr = os.Stderr
		sh.Stdin = os.Stdin
		sh.Run()
	},
}

var appThemeCmd = &cobra.Command{
	Use:   "theme [theme]",
	Short: "Get/Set theme",
	Long:  `Set or Set the theme for fig.`,
	Args:  cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, arg []string) {
		settings, err := settings.Load()
		if err != nil {
			fmt.Println("Error loading settings:", err)
		}

		if len(arg) == 0 {
			fmt.Println(settings.Get("autocomplete.theme"))
			return
		}

		bulitinTheme := []string{"dark", "light"}

		usr, err := user.Current()
		if err != nil {
			fmt.Println(err)
			return
		}

		data, err := ioutil.ReadFile(fmt.Sprintf("%s/.fig/themes/%s.json", usr.HomeDir, arg[0]))
		if err != nil {
			// If builtin theme, just set it
			for _, t := range bulitinTheme {
				if t == arg[0] {
					fmt.Println("› Switching to theme '" + lipgloss.NewStyle().Bold(true).Render(arg[0]) + "'")
					settings.Save()
					return
				}
			}

			fmt.Printf("'%s' does not exist in ~/.fig/themes/\n", arg[0])
			return
		}

		var theme map[string]interface{}
		err = json.Unmarshal(data, &theme)

		if err != nil {
			fmt.Println("Error parsing theme json")
			return
		}

		author := theme["author"]
		authorName := author.(map[string]interface{})["name"]
		twitter := author.(map[string]interface{})["twitter"]
		github := author.(map[string]interface{})["github"]

		byLine := fmt.Sprintf("› Switching to theme '" + lipgloss.NewStyle().Bold(true).Render(arg[0]) + "'")
		if authorName != nil {
			byLine += fmt.Sprintf(" by %s", lipgloss.NewStyle().Bold(true).Render(authorName.(string)))
		}

		settings.Set("autocomplete.theme", arg[0])
		settings.Save()

		fmt.Println()
		fmt.Println(byLine)
		if twitter != nil {
			fmt.Println("  🐦 " + lipgloss.NewStyle().Foreground(lipgloss.Color("#1DA1F2")).Render(twitter.(string)))
		}
		if github != nil {
			fmt.Println("  💻 " + lipgloss.NewStyle().Underline(true).Render("github.com/"+github.(string)))
		}
		fmt.Println()
	},
}

var appUpgradeCmd = &cobra.Command{
	Use:   "install-and-upgrace",
	Short: "Install and upgrade fig",
	Args:  cobra.MaximumNArgs(1),
	Run: func(cmd *cobra.Command, arg []string) {
		if len(arg) == 1 {
			fmt.Println("Tag is", arg[0])
		}

		// Get unix epoch time
		t := time.Now().Unix()

		// Get user
		usr, err := user.Current()
		if err != nil {
			fmt.Println(err)
			return
		}

		// Create ~/.fig directory if it doesn't exist
		if _, err := os.Stat(usr.HomeDir + "/.fig"); os.IsNotExist(err) {
			os.MkdirAll(usr.HomeDir+"/.fig", 0755)
		}

		// delete binary artifacts to ensure ad-hoc code signature works for arm64 binaries on M1
		files, err := ioutil.ReadDir(usr.HomeDir + "/.fig/bin")
		if err != nil {
			fmt.Println(err)
			return
		}

		for _, f := range files {
			if strings.Contains(f.Name(), "figterm") {
				os.Remove(usr.HomeDir + "/.fig/bin/" + f.Name())
			}
		}

		os.Remove(usr.HomeDir + "/.fig/bin/fig_callback")
		os.Remove(usr.HomeDir + "/.fig/bin/fig_get_shell")

		if len(arg) >= 1 && arg[0] == "local" {
			cp := exec.Command("cp", "-R", "./", usr.HomeDir+"/.fig")
			cp.Stdout = os.Stdout
			cp.Stderr = os.Stderr
			cp.Stdin = os.Stdin
			cp.Run()
		}

		// Make files and folders that the user can edit (that aren't overridden by above)
		os.MkdirAll(usr.HomeDir+"/.fig/bin", 0755)
		os.MkdirAll(usr.HomeDir+"/.fig/zle", 0755)
		os.MkdirAll(usr.HomeDir+"/.fig/autocomplete", 0755)

		os.MkdirAll(usr.HomeDir+"/.fig/user/aliases", 0755)
		os.MkdirAll(usr.HomeDir+"/.fig/user/apps", 0755)
		os.MkdirAll(usr.HomeDir+"/.fig/user/autocomplete", 0755)
		os.MkdirAll(usr.HomeDir+"/.fig/user/aliases", 0755)

		// rename figterm binaries to mirror supported shell
		// copy binaries on install to avoid issues with file permissions at runtime
		figterm := "/Applications/Fig.app/Contents/MacOS/figterm"

		// copy figterm to ~/.fig/bin
		exec.Command("cp", "-p", figterm, usr.HomeDir+"/.fig/bin/zsh\\ \\(figterm\\)").Run()
		exec.Command("cp", "-p", figterm, usr.HomeDir+"/.fig/bin/bash\\ \\(figterm\\)").Run()
		exec.Command("cp", "-p", figterm, usr.HomeDir+"/.fig/bin/fish\\ \\(figterm\\)").Run()

		// If ~/.fig/settings.json does not exist, create it
		if _, err := os.Stat(usr.HomeDir + "/.fig/settings.json"); os.IsNotExist(err) {
			settings := settings.New()
			settings.Save()
		}

		// If ~/.fig/user/aliases/_myaliases.sh does not exist, create it
		if _, err := os.Stat(usr.HomeDir + "/.fig/user/aliases/_myaliases.sh"); os.IsNotExist(err) {
			ioutil.WriteFile(usr.HomeDir+"/.fig/user/aliases/_myaliases.sh", []byte(""), 0755)
		}

		// If ~/.fig/user/figpath.sh does not exist, create it
		if _, err := os.Stat(usr.HomeDir + "/.fig/user/figpath.sh"); os.IsNotExist(err) {
			ioutil.WriteFile(usr.HomeDir+"/.fig/user/figpath.sh", []byte(""), 0755)
		}

		// Determine user's login shell by explicitly reading from "/Users/$(whoami)"
		// rather than ~ to handle rare cases where these are different.
		dsclExec, err := exec.Command("dscl", ".", "-read", "/Users/"+usr.Username, "UserShell").Output()
		shell := strings.TrimSpace(string(dsclExec))
		shellName := strings.Split(shell, " ")[1]

		if err != nil {
			fmt.Println("Error: ", err)
			return
		}

		exec.Command("defaults", "write", "com.mschrage.fig", "userShell", shell).Run()

		shellPath, _ := exec.Command(shellName, "-li", "-c", "/usr/bin/env | /usr/bin/grep '^PATH=' | /bin/cat | /usr/bin/sed 's|PATH=||g'").Output()

		settings, _ := settings.Load()

		settings.Set("userShell", shellName)
		settings.Set("pty.path", string(shellPath))
		settings.Set("autocomplete.addStatusToTerminalTitle", false)

		settings.Save()

		// TODO: Replace with native implementation
		figcli := "/Applications/Fig.app/Contents/MacOS/figcli"
		exec.Command(figcli, "settings:init").Run()

		// Onboarding

		// If ~/.fig/user/config does not exist, create it
		if _, err := os.Stat(usr.HomeDir + "/.fig/user/config"); os.IsNotExist(err) {
			ioutil.WriteFile(usr.HomeDir+"/.fig/user/config", []byte(""), 0755)
		}

		// Load ~/.fig/user/config
		config, err := ioutil.ReadFile(usr.HomeDir + "/.fig/user/config")
		if err != nil {
			fmt.Println(err)
			return
		}

		// If this is first download, mark download time as now.
		if !strings.Contains(string(config), "DOWNLOAD_TIME") {
			config = append(config, []byte(fmt.Sprintf("DOWNLOAD_TIME=%d\n", t))...)
		}

		// Create last_update if it doesn't exist and mark last update as now.
		if !strings.Contains(string(config), "LAST_UPDATE") {
			config = append(config, []byte(fmt.Sprintf("LAST_UPDATE=%d\n", t))...)
		} else {
			re := regexp.MustCompile("LAST_UPDATE=(.*)")
			config = re.ReplaceAll(config, []byte(fmt.Sprintf("LAST_UPDATE=%d", t)))
		}

		// Add config variables to ~/.fig/settings.json
		addConfVar := func(line string) {
			if !strings.Contains(string(config), line) {
				config = append(config, []byte(line+"\n")...)
			}
		}

		addConfVar("FIG_LOGGED_IN")
		addConfVar("FIG_ONBOARDING")
		addConfVar("DONT_SHOW_DRIP")
		for _, num := range []string{"ONE", "TWO", "THREE", "FOUR", "FIVE", "SIX", "SEVEN"} {
			addConfVar("DRIP_" + num)
		}

		// Write config back to ~/.fig/user/config
		ioutil.WriteFile(usr.HomeDir+"/.fig/user/config", config, 0755)

		// hotfix for infinite looping when writing "☑ fig" title to a tty backed by figterm
		exec.Command("defaults", "write", "com.mschrage.fig", "addIndicatorToTitlebar", "false").Run()

		exec.Command("sh", "~/.fig/tools/install_integrations.sh").Run()

		fmt.Println("success")
	},
}

var appSetPath = &cobra.Command{
	Use:   "set-path",
	Short: "Set the path to the fig executable",
	Long:  `Set the path to the fig executable`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Printf("\n  Setting $PATH variable in Fig pseudo-terminal...\n\n\n")

		// Get the users $PATH
		path := os.Getenv("PATH")

		// Load ~/.fig/settings.json and set the path
		settings, err := settings.Load()
		if err != nil {
			fmt.Println("Error: ", err)
			return
		}

		settings.Set("pty.path", path)

		// Trigger update of ENV in PTY
		pty, err := diagnostics.GetTty()
		if err != nil {
			fmt.Println("Error: ", err)
			return
		}

		hook, _ := fig_ipc.CreateInitHook(os.Getppid(), pty)
		fig_ipc.SendHook(hook)
	},
}

var appUninstallCmd = &cobra.Command{
	Use:   "uninstall",
	Short: "Uninstall Fig",
	Long:  `Uninstall Fig`,
	Run: func(cmd *cobra.Command, args []string) {
		sh := exec.Command("bash", "-c", "~/.fig/tools/uninstall-script.sh")
		sh.Stdout = os.Stdout
		sh.Stderr = os.Stderr
		sh.Stdin = os.Stdin
		sh.Run()
	},
}

var appRunningCmd = &cobra.Command{
	Use:   "running",
	Short: "Gets the status if Fig is running",
	Run: func(cmd *cobra.Command, args []string) {
		appInfo, err := diagnostics.GetAppInfo()
		if err != nil {
			return
		}

		if appInfo.IsRunning() {
			fmt.Println("1")
		} else {
			fmt.Println("0")
		}
	},
}
