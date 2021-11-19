package uninstall

import (
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCommandUninstall() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "uninstall",
		Short: "Uninstall Fig",
		Long:  `Uninstall Fig`,
		Run: func(cmd *cobra.Command, args []string) {
			// Use uninstall script for time being
			installCmd := exec.Command("sh", "-c", "~/.fig/tools/uninstall-script.sh")
			installCmd.Stdout = os.Stdout
			installCmd.Stderr = os.Stderr
			installCmd.Run()

			// // Get user
			// usr, err := user.Current()
			// if err != nil {
			// 	fmt.Println("Error: ", err)
			// 	return
			// }

			// fmt.Println("Deleting .fig folder & completion specs")
			// os.RemoveAll(usr.HomeDir + "/.fig")

			// fmt.Println("Delete backup Fig CLI")
			// os.RemoveAll("/usr/local/bin/fig")

			// fmt.Println("Deleting WKWebViewCache")
			// fig_ipc.RunResetCacheCommand()

			// fmt.Println("Deleting fig defaults & preferences")
			// savedId, err := exec.Command("defaults", "read", "com.mschrage.fig", "uuid").Output()
			// if err != nil {
			// 	fmt.Println("Error: ", err)
			// 	return
			// }

			// exec.Command("defaults", "delete", "com.mschrage.fig").Run()
			// exec.Command("defaults", "delete", "com.mschrage.fig.shared").Run()
			// exec.Command("defaults", "write", "com.mschrage.fig", "uuid", string(savedId)).Run()

			// fmt.Println("Remove iTerm integration (if set up)")
			// os.Remove(usr.HomeDir + "/Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.py")
			// os.Remove(usr.HomeDir + "/.config/iterm2/AppSupport/Scripts/AutoLaunch/fig-iterm-integration.py")
			// os.Remove(usr.HomeDir + "/Library/Application Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt")

			// fmt.Println("Remove VSCode integration (if set up)")
			// files, _ := filepath.Glob(usr.HomeDir + "/.vscode/extensions/withfig.fig-*")
			// for _, file := range files {
			// 	os.RemoveAll(file)
			// }
			// files, _ = filepath.Glob(usr.HomeDir + "/.vscode-insiders/extensions/withfig.fig-*")
			// for _, file := range files {
			// 	os.RemoveAll(file)
			// }

			// fmt.Println("Remove fish integration...")
			// os.Remove(usr.HomeDir + "/.config/fish/conf.d/fig.fish")

			// fmt.Println("Removing fig.sh setup from .profile, .zprofile, .zshrc, .bash_profile, and .bashrc")
			// for _, file := range []string{".profile", ".zprofile", ".zshrc", ".bash_profile", ".bashrc"} {
			// 	if _, err := os.Stat(usr.HomeDir + "/" + file); err == nil {
			// 		// File exists
			// 		lines, err := os.ReadFile(usr.HomeDir + "/" + file)
			// 		if err != nil {
			// 			fmt.Println("Error: ", err)
			// 			return
			// 		}

			// 		lines = bytes.Replace(lines, []byte("#### FIG ENV VARIABLES ####\n"), []byte(""), -1)
			// 		lines = bytes.Replace(lines, []byte("# Please make sure this block is at the start of this file.\n"), []byte(""), -1)
			// 		lines = bytes.Replace(lines, []byte("[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh\n"), []byte(""), -1)
			// 		lines = bytes.Replace(lines, []byte("[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh\n"), []byte(""), -1)
			// 		lines = bytes.Replace(lines, []byte("# Please make sure this block is at the end of this file.\n"), []byte(""), -1)
			// 		lines = bytes.Replace(lines, []byte("#### END FIG ENV VARIABLES ####\n"), []byte(""), -1)

			// 		os.WriteFile(usr.HomeDir+"/"+file, lines, 0755)
			// 	}
			// }

			// fmt.Println("Removing fish integration")
			// if _, err := os.Stat(usr.HomeDir + "/.config/fish/config.fish"); err == nil {
			// 	// File exists
			// 	lines, err := os.ReadFile(usr.HomeDir + "/.config/fish/config.fish")
			// 	if err != nil {
			// 		fmt.Println("Error: ", err)
			// 		return
			// 	}

			// 	fishInstall := fmt.Sprintf("contains %s/.fig/bin %s; or set -Ua fish_user_paths %s/.fig/bin\n",
			// 		usr.HomeDir, os.Getenv("fish_user_paths"), usr.HomeDir)

			// 	lines = bytes.Replace(lines, []byte(fishInstall), []byte(""), -1)

			// 	os.WriteFile(usr.HomeDir+"/.config/fish/config.fish", lines, 0755)
			// }
			// os.Remove(usr.HomeDir + "/.config/fish/conf.d/fig.fish")

			// fmt.Println("Remove Hyper plugin, if it exists")
			// if _, err := os.Stat(usr.HomeDir + "/.hyper.js"); err == nil {
			// 	lines, err := os.ReadFile(usr.HomeDir + "/.hyper.js")
			// 	if err != nil {
			// 		fmt.Println("Error: ", err)
			// 		return
			// 	}

			// 	lines = bytes.Replace(lines, []byte("\"fig-hyper-integration\","), []byte(""), -1)
			// 	lines = bytes.Replace(lines, []byte("\"fig-hyper-integration\""), []byte(""), -1)

			// 	os.WriteFile(usr.HomeDir+"/.hyper.js", lines, 0755)
			// }

			// fmt.Println("Finished removing fig resources. You may now delete the Fig app by moving it to the Trash.")

			// os.RemoveAll(usr.HomeDir + "/Library/Input Methods/FigInputMethod.app")
			// os.RemoveAll(usr.HomeDir + "/Applications/Fig.app")

			// fig_ipc.QuitCommand()
		},
	}

	return cmd
}
