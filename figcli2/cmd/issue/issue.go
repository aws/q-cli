package issue

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	"fmt"
	"net/url"
	"os/exec"
	"regexp"
	"runtime"
	"strings"

	"github.com/spf13/cobra"
)

func NewCmdIssue() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "issue",
		Short: "Create a new GitHub issue",
		Long:  "Create a new GitHub issue in withfig/fig.",
		Annotations: map[string]string{
			"figcli.command.categories": "Common",
		},
		Args: cobra.ArbitraryArgs,
		Run: func(cmd *cobra.Command, arg []string) {
			text := strings.Join(arg, " ")

			assignees := []string{"mschrage"}

			// If args include "cli", then add the figcli team to the assignees
			if regexp.MustCompile(`(?i)cli`).MatchString(text) {
				assignees = append(assignees, "grant0417")
			}

			// If args include "figterm", then add the figterm team to the assignees
			if regexp.MustCompile(`(?i)figterm`).MatchString(text) {
				assignees = append(assignees, "sullivan-sean")
			}

			osName := runtime.GOOS
			osArch := runtime.GOARCH
			fmt.Println(osName, osArch)

			var body strings.Builder

			body.WriteString("### Description:\n> Please include a detailed description of the issue (and an image or screen recording, if applicable)\n\n")

			if len(text) > 0 {
				body.WriteString(text)
			}

			body.WriteString("\n\n### Details:\n|macOS|Fig|Shell|\n|-|-|-|\n")

			macOsVersion, _ := diagnostics.GetMacOsVersion()
			figVersion, _ := diagnostics.GetFigVersion()
			shell, _ := fig_ipc.GetShell()

			body.WriteString(fmt.Sprintf("|%s|%s|%s|\n", macOsVersion, figVersion, shell))

			body.WriteString("<details><summary><code>fig diagnostic</code></summary>\n<p>\n<pre>")

			diagnostic := diagnostics.Summary()

			body.WriteString(diagnostic)

			body.WriteString("</pre>\n</p>\n</details>")

			fmt.Println(body.String())

			fmt.Println("→ Opening GitHub...")
			exec.Command("open",
				fmt.Sprintf("https://github.com/withfig/fig/issues/new?labels=bug&assignees=%s&body=%s",
					strings.Join(assignees, ","),
					url.QueryEscape(body.String())),
			).Run()
		},
	}

	return cmd
}
