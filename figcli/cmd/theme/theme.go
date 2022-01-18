package theme

import (
	"encoding/json"
	"fig-cli/settings"
	"fmt"
	"os"
	"os/user"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

var BuiltinThemes []string = []string{"dark", "light", "system"}

type Author struct {
	Name    string `json:"name"`
	Twitter string `json:"twitter"`
	Github  string `json:"github"`
}

type Theme struct {
	Author  Author `json:"author"`
	Version string `json:"version"`
}

func setTheme(themeStr string) (string, error) {
	settings, err := settings.Load()
	if err != nil {
		return "", fmt.Errorf("can not load settings: %s", err)
	}

	usr, err := user.Current()
	if err != nil {
		return "", fmt.Errorf("can not getting current user: %s", err)
	}
	data, err := os.ReadFile(fmt.Sprintf("%s/.fig/themes/%s.json", usr.HomeDir, themeStr))
	if err != nil {
		// If builtin theme, just set it
		for _, t := range BuiltinThemes {
			if t == themeStr {
				settings.Set("autocomplete.theme", themeStr)
				settings.Save()
				return fmt.Sprintf("› Switching to theme '" + lipgloss.NewStyle().Bold(true).Render(themeStr) + "'"), nil
			}
		}

		return fmt.Sprintf("'%s' does not exist in ~/.fig/themes/\n", themeStr), nil
	}

	var theme Theme
	err = json.Unmarshal(data, &theme)

	if err != nil {
		return "", fmt.Errorf("error parsing theme json: %s", err)
	}

	authorName := theme.Author.Name
	twitter := theme.Author.Twitter
	github := theme.Author.Github

	byLine := fmt.Sprintf("› Switching to theme '" + lipgloss.NewStyle().Bold(true).Render(themeStr) + "'")
	if authorName != "" {
		byLine += fmt.Sprintf(" by %s", lipgloss.NewStyle().Bold(true).Render(authorName))
	}

	settings.Set("autocomplete.theme", themeStr)

	err = settings.Save()
	if err != nil {
		return "", fmt.Errorf("can not save settings: %s", err)
	}

	output := "\n"
	output += byLine + "\n"
	if twitter != "" {
		output += "  🐦 " + lipgloss.NewStyle().Foreground(lipgloss.Color("#1DA1F2")).Render(twitter) + "\n"
	}
	if github != "" {
		output += "  💻 " + lipgloss.NewStyle().Underline(true).Render("github.com/"+github) + "\n"
	}

	return output, nil
}

func getTheme() (string, error) {
	settings, err := settings.Load()
	if err != nil {
		return "", fmt.Errorf("can not load settings: %s", err)
	}

	theme := settings.Get("autocomplete.theme")
	if theme == nil {
		return "dark", nil
	}

	return theme.(string), nil
}

func NewCmdTheme() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "theme [theme]",
		Short: "Get/Set theme",
		Long:  `Set or Set the theme for fig.`,
		Args:  cobra.MaximumNArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) == 0 {
				theme, err := getTheme()
				if err != nil {
					fmt.Println(err)
					os.Exit(1)
				}
				fmt.Println(theme)
			} else {
				output, err := setTheme(args[0])
				if err != nil {
					fmt.Println(err)
					os.Exit(1)
				}
				fmt.Println(output)
			}
		},
	}

	return cmd
}
