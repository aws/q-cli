package careers

import (
	"fig-cli/logging"
	"fmt"
	"os/exec"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdCareers() *cobra.Command {
	cmd := &cobra.Command{
		Use: "careers",
		Aliases: []string{
			"career",
			"apply",
			"dir",
			"curl",
			"runbooks",
			"psql",
			"git",
			"google",
		},
		Run: func(cmd *cobra.Command, args []string) {
			fmt.Println(lipgloss.NewStyle().Width(60).Padding(2).Render("You probably came here as you saw one of our demos!\n\n" +
				"We've deliberately hidden all Fig apps other than autocomplete for the time being. Our focus right now is stabilizing our foundation and making sure you're having a fantastic experience on autocomplete.\n\n" +
				"If you'd like to come join us and help build these apps, check out " + lipgloss.NewStyle().Underline(true).Render("fig.io/jobs")))
			if err := exec.Command("open", "https://fig.io/jobs").Run(); err != nil {
				logging.Log("Unable to open link", err.Error())
			}
		},
	}

	return cmd
}
