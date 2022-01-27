package invite

import (
	"fmt"
	"io"
	"net/http"
	"os/exec"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

func NewCmdInvite() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "invite",
		Short: "Invite friends to Fig",
		Long:  "Invite up to 5 friends & teammates to Fig",
		Run: func(cmd *cobra.Command, arg []string) {
			email, err := exec.Command("defaults", "read", "com.mschrage.fig", "userEmail").Output()

			emailStr := strings.TrimSpace(string(email))
			if err != nil || emailStr == "" {
				fmt.Println()
				fmt.Println(
					lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#ff0000")).Render("Error") +
						lipgloss.NewStyle().Bold(true).Render(": It does not seem like you are logged into Fig."),
				)
				fmt.Println()
				fmt.Println(
					"Run " +
						lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("5")).Render("fig user logout") +
						" then follow the prompts to log back. Then try again",
				)
				fmt.Println()
				return
			}

			res, err := http.Get("https://api.fig.io/waitlist/get-referral-link-from-email/" + strings.TrimSpace(string(email)))

			if err != nil {
				fmt.Println()
				fmt.Println(
					lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#ff0000")).Render("Error") +
						lipgloss.NewStyle().Bold(true).Render(": It does not seem like you are able to reach the internet."),
				)
				fmt.Println()
				return
			}

			body, err := io.ReadAll(res.Body)
			res.Body.Close()

			if res.StatusCode > 299 || err != nil {
				fmt.Println()
				fmt.Println(
					lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("#ff0000")).Render("Error") +
						lipgloss.NewStyle().Bold(true).Render(": We can't find a referral code for this email address: "+emailStr),
				)
				fmt.Println()
				fmt.Println(
					lipgloss.NewStyle().Underline(true).Render("Are you sure you are logged in correctly?"),
				)
				fmt.Println()
				fmt.Println(
					"Run " +
						lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("5")).Render("fig user logout") +
						" then follow the prompts to log back. Then try again",
				)
				fmt.Println()
				fmt.Println(
					"If you think there is a mistake, please contact " +
						lipgloss.NewStyle().Underline(true).Render("hello@fig.io"),
				)
				fmt.Println()
				return
			}

			pbcopyCmd := exec.Command("pbcopy")
			pbcopyStdin, _ := pbcopyCmd.StdinPipe()

			pbcopyCmd.Start()
			io.Copy(pbcopyStdin, strings.NewReader(string(body)))
			pbcopyStdin.Close()

			fmt.Println()
			fmt.Println(lipgloss.NewStyle().Bold(true).Render("Thank you for sharing Fig."))
			fmt.Println()
			fmt.Println("> " + lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("5")).Render(string(body)))
			fmt.Println("  Your referral link has been copied to the clipboard.")
			fmt.Println()
		},
	}

	return cmd
}
