package source

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"os/user"
	"path/filepath"

	"github.com/spf13/cobra"
)

func DownloadAndSave(url string, filePath string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	out, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

func NewCmdSource() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "source",
		Short: "Source dotfiles",
		Run: func(cmd *cobra.Command, arg []string) {
			// Get the current user 
			user, err := user.Current()
			if err != nil {
				fmt.Printf("Error getting current user: %s", err)
				os.Exit(1)
			}

			// Directory to save the dotfiles
			dotfiles_dir := filepath.Join(user.HomeDir, ".fig", "user", "dotfiles")

			// Make dotfiles directory
			if err := os.MkdirAll(dotfiles_dir, 0755); err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			// TODO: Determine the url/api to download the dotfiles
			bash_url := "https://gist.githubusercontent.com/grant0417/916e80ae32717eeec18d2c7a50a13192/raw/9e0e44b994a30447d448b80063efb04f7be87d3c/gistfile1.txt"
			zsh_url := "https://gist.githubusercontent.com/grant0417/916e80ae32717eeec18d2c7a50a13192/raw/9e0e44b994a30447d448b80063efb04f7be87d3c/gistfile1.txt"
			fish_url := "https://gist.githubusercontent.com/grant0417/916e80ae32717eeec18d2c7a50a13192/raw/9e0e44b994a30447d448b80063efb04f7be87d3c/gistfile1.txt"

			// Filenames of each dotfile
			bash_location := filepath.Join(dotfiles_dir, "managed.bash")
			zsh_location := filepath.Join(dotfiles_dir, "managed.zsh")
			fish_location := filepath.Join(dotfiles_dir, "managed.fish")

			// Download the dotfiles to the dotfiles directory

			err = DownloadAndSave(bash_url, bash_location)
			if err != nil {
				fmt.Printf("Error downloading bash: %s", err)
				os.Exit(1)
			}

			err = DownloadAndSave(zsh_url, zsh_location)
			if err != nil {
				fmt.Printf("Error downloading zsh: %s", err)
				os.Exit(1)
			}

			err = DownloadAndSave(fish_url, fish_location)
			if err != nil {
				fmt.Printf("Error downloading fish: %s", err)
				os.Exit(1)
			}

			fmt.Printf("\n→ Dotfiles synced\n\n")
		},
	}

	return cmd
}
