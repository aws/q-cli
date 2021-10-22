package cmd

import (
	"fig-cli/diagnostics"
	"fmt"
	"runtime"
	"strings"

	"github.com/spf13/cobra"
)

func init() {
	rootCmd.AddCommand(issueCmd)
}

var issueCmd = &cobra.Command{
	Use:   "issue",
	Short: "Create a new GitHub issue",
	Long:  "Create a new GitHub issue in withfig/fig.",
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},
	Args: cobra.ArbitraryArgs,
	Run: func(cmd *cobra.Command, arg []string) {

		osName := runtime.GOOS
		osArch := runtime.GOARCH
		fmt.Println(osName, osArch)

		var body strings.Builder

		body.WriteString("### Description:\n> Please include a detailed description of the issue (and an image or screen recording, if applicable)\n\n")

		if len(arg) > 0 {
			body.WriteString(strings.Join(arg, " "))
		}

		body.WriteString("\n\n\n\n### Details:\n|macOS|Fig|Shell|\n|-|-|-|\n")

		macOsVersion, _ := diagnostics.GetMacOsVersion()
		figVersion, _ := diagnostics.GetFigVersion()
		shell, _ := diagnostics.GetShell()

		body.WriteString(fmt.Sprintf("|%s|%s|%s|\n", macOsVersion, figVersion, shell))

		body.WriteString(fmt.Sprintf("|%s|%s|%s|\n", osName, "fig", "bash"))
		body.WriteString("<details><summary><code>fig diagnostic</code></summary>\n<p>\n<pre>")

		//\(Diagnostic.summary.trimmingCharacters(in: .whitespacesAndNewlines))

		body.WriteString("</pre>\n</p>\n</details>")

		fmt.Println(body.String())

		// fmt.Println("→ Opening GitHub...")
		// exec.Command("open", "https://github.com/withfig/fig/issues/new?labels=bug&assignees=mattschrage&body="+url.QueryEscape(body.String())).Run()
	},
}
