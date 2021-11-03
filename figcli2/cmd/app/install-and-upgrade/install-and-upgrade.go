package installandupgrade

import (
	"bytes"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/settings"
	"fmt"
	"os"
	"os/exec"
	"os/user"
	"regexp"
	"strings"
	"time"

	"github.com/spf13/cobra"
)

func backupDotfile(path string) error {
	usr, err := user.Current()
	if err != nil {
		return err
	}

	// Backup dotfile by copy
	data, err := ioutil.ReadFile(path)
	if os.IsNotExist(err) {
		return nil
	}

	if err != nil {
		return err
	}

	// Make directory if it doesn't exist at ~/.fig
	if _, err := os.Stat(usr.HomeDir + "/.fig.dotfiles.bak/"); os.IsNotExist(err) {
		os.MkdirAll(usr.HomeDir+"/.fig.dotfiles.bak/", 0755)
	}

	err = ioutil.WriteFile(usr.HomeDir+"/.fig.dotfiles.bak/"+path, data, 0644)
	if err != nil {
		return err
	}

	return nil
}

func installIntegrations() {
	// Add the fig.sh to your profiles so it can be sourced on new terminal window load

	usr, err := user.Current()
	if err != nil {
		fmt.Println(err)
		return
	}

	// Make sure one of [.bash_profile|.bash_login|.profile] exists to ensure fig
	// is sourced on login shells. We choose .profile to be as minimally
	// disruptive to existing user set up as possible.
	// https://superuser.com/questions/320065/bashrc-not-sourced-in-iterm-mac-os-x
	if _, err := os.Stat(usr.HomeDir + "/.profile"); os.IsNotExist(err) {
		os.Create(usr.HomeDir + "/.profile")
	}

	// replace old sourcing in profiles
	for _, profile := range []string{".profile", ".zprofile", ".bash_profile"} {
		data, err := ioutil.ReadFile(usr.HomeDir + "/" + profile)
		if os.IsNotExist(err) {
			continue
		}

		if err != nil {
			fmt.Println(err)
			return
		}

		newBytes := bytes.Replace(data, []byte("~/.fig/exports/env.sh"), []byte("~/.fig/fig.sh"), -1)

		err = ioutil.WriteFile(usr.HomeDir+"/"+profile, newBytes, 0644)
		if err != nil {
			fmt.Println(err)
			return
		}
	}

	// Create .zshrc/.bashrc regardless of whether it exists or not
	for _, profile := range []string{".zshrc", ".bashrc"} {
		if _, err := os.Stat(usr.HomeDir + "/" + profile); os.IsNotExist(err) {
			os.Create(usr.HomeDir + "/" + profile)
		}
	}

	// Don't modify files until all are backed up.
	for _, profile := range []string{".profile", ".zprofile", ".bash_profile", ".bash_login", ".bashrc", ".zshrc"} {
		if err := backupDotfile(profile); err != nil {
			fmt.Println(err)
			return
		}
	}

	// Install fig.sh
	for _, profile := range []string{".profile", ".zprofile", ".bash_profile", ".bash_login", ".bashrc", ".zshrc"} {
		data, err := ioutil.ReadFile(usr.HomeDir + "/" + profile)
		if os.IsNotExist(err) {
			continue
		}

		if err != nil {
			fmt.Println(err)
			return
		}

		newBytes := []byte("\n#### FIG ENV VARIABLES ####\n# Please make sure this block is at the start of this file.\n[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh\n#### END FIG ENV VARIABLES ####\n")
		newBytes = append(newBytes, data...)
		newBytes = append(newBytes, []byte("\n#### FIG ENV VARIABLES ####\n# Please make sure this block is at the end of this file.\n[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh\n#### END FIG ENV VARIABLES ####\n")...)

		err = ioutil.WriteFile(usr.HomeDir+"/"+profile, newBytes, 0644)
		if err != nil {
			fmt.Println(err)
			return
		}
	}

	// Handle fish separately
	if _, err := os.Stat(usr.HomeDir + "/.config/fish/conf.d"); os.IsNotExist(err) {
		os.MkdirAll(usr.HomeDir+"/.config/fish/conf.d", 0755)
	}

	// link pre.fish and post.fish
	// Use 00_/99_ prefix to load script earlier/later in fish startup.
	os.Symlink(usr.HomeDir+"/.fig/shell/pre.fish", usr.HomeDir+"/.config/fish/conf.d/00_pre.fish")
	os.Symlink(usr.HomeDir+"/.fig/shell/post.fish", usr.HomeDir+"/.config/fish/conf.d/99_post.fish")

	// Remove deprecated fish config file.
	if _, err := os.Stat(usr.HomeDir + "/.config/fish/config.fish"); os.IsExist(err) {
		os.Remove(usr.HomeDir + "/.config/fish/conf.d/fig.fish")
	}
}

func NewCmdInstallAndUpgrade() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "install-and-upgrade",
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
			files, err := os.ReadDir(usr.HomeDir + "/.fig/bin")
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
				os.WriteFile(usr.HomeDir+"/.fig/user/aliases/_myaliases.sh", []byte(""), 0755)
			}

			// If ~/.fig/user/figpath.sh does not exist, create it
			if _, err := os.Stat(usr.HomeDir + "/.fig/user/figpath.sh"); os.IsNotExist(err) {
				os.WriteFile(usr.HomeDir+"/.fig/user/figpath.sh", []byte(""), 0755)
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

			// Restart settings listener
			fig_ipc.RestartSettingsListenerCommand()

			// Onboarding

			// If ~/.fig/user/config does not exist, create it
			if _, err := os.Stat(usr.HomeDir + "/.fig/user/config"); os.IsNotExist(err) {
				os.WriteFile(usr.HomeDir+"/.fig/user/config", []byte(""), 0755)
			}

			// Load ~/.fig/user/config
			config, err := os.ReadFile(usr.HomeDir + "/.fig/user/config")
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
			os.WriteFile(usr.HomeDir+"/.fig/user/config", config, 0755)

			// hotfix for infinite looping when writing "☑ fig" title to a tty backed by figterm
			exec.Command("defaults", "write", "com.mschrage.fig", "addIndicatorToTitlebar", "false").Run()

			installIntegrations()

			fmt.Println("success")
		},
	}

	return cmd
}
