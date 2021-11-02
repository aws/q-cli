package theme

import (
	"encoding/json"
	"fig-cli/settings"
	"fmt"
	"io/ioutil"
	"os/user"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewThemeCmd() *cobra.Command {
	cmd := &cobra.Command{
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

	return cmd
}
