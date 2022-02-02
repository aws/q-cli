package root

import (
	"fig-cli/cmd/alpha"
	"fig-cli/cmd/app"
	"fig-cli/cmd/app/onboarding"
	"fig-cli/cmd/careers"
	"fig-cli/cmd/community"
	contrib "fig-cli/cmd/contributors"
	"fig-cli/cmd/debug"
	"fig-cli/cmd/debug/diagnostic"
	"fig-cli/cmd/dev"
	"fig-cli/cmd/docs"
	"fig-cli/cmd/doctor"
	"fig-cli/cmd/hook"
	legacy_focus_changed "fig-cli/cmd/hook/keyboard-focus-changed"
	"fig-cli/cmd/integrations"
	"fig-cli/cmd/invite"
	"fig-cli/cmd/issue"
	"fig-cli/cmd/launch"
	"fig-cli/cmd/quit"
	"fig-cli/cmd/restart"
	"fig-cli/cmd/settings"
	"fig-cli/cmd/source"
	"fig-cli/cmd/specs"
	"fig-cli/cmd/theme"
	"fig-cli/cmd/tips"
	"fig-cli/cmd/tweet"
	"fig-cli/cmd/update"
	"fig-cli/cmd/user"
	"fig-cli/cmd/user/logout"
	"fig-cli/diagnostics"
	"fig-cli/logging"
	"fmt"
	"os"
	"strings"

	genFigSpec "github.com/withfig/autocomplete-tools/packages/cobra"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

const TextWidth = 60

var rootCmd = &cobra.Command{
	Use:   "fig",
	Short: "CLI to interact with Fig",
	Version: func() string {
		version, _ := diagnostics.GetFigVersion()
		build, _ := diagnostics.GetFigBuild()
		return fmt.Sprintf("%s B%s", version, build)
	}(),
}

func Execute() {
	rootCmd.SetHelpFunc(func(cmd *cobra.Command, args []string) {
		// Map from subcommand name to the cobra.Command that represents it
		commands := make(map[string]*cobra.Command)
		for _, cmd := range cmd.Commands() {
			commands[cmd.Name()] = cmd
		}

		// Commands to show in help
		commandKeys := [](string){
			"doctor",
			"settings",
			"issue",
			"tweet",
			"update",
		}

		if !cmd.HasParent() {
			// Help page for root command

			println(lipgloss.NewStyle().
				Bold(true).
				PaddingTop(1).
				PaddingLeft(2).
				Width(TextWidth).
				Align(lipgloss.Left).
				Render(`███████╗██╗ ██████╗
██╔════╝██║██╔════╝
█████╗  ██║██║  ███╗
██╔══╝  ██║██║   ██║
██║     ██║╚██████╔╝
╚═╝     ╚═╝ ╚═════╝ CLI`))

			fmt.Println(lipgloss.
				NewStyle().
				Padding(1, 1, 0).
				Width(TextWidth).
				Foreground(lipgloss.AdaptiveColor{Light: "236", Dark: "248"}).
				Render(
					lipgloss.NewStyle().
						Bold(true).
						Render(`Usage: `) +
						lipgloss.NewStyle().
							Italic(true).
							Foreground(lipgloss.AdaptiveColor{Light: "236", Dark: "248"}).
							Render(`fig [subcommand]`),
				))

			fmt.Println(lipgloss.
				NewStyle().
				Padding(0, 1).
				Width(TextWidth).
				Render(
					lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "236", Dark: "248"}).Render("Running ") +
						lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "236", Dark: "248"}).Bold(true).Render("fig") +
						lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{Light: "236", Dark: "248"}).Render(" will soon take you to mission control")))

			fmt.Println(lipgloss.
				NewStyle().
				Padding(1, 1, 0).
				Foreground(lipgloss.Color("5")).
				// Border(lipgloss.RoundedBorder(), true, true).
				Width(TextWidth).
				Bold(true).
				Render("Common Subcommands"))

			var sb strings.Builder

			for _, key := range commandKeys {
				sb.WriteString(lipgloss.NewStyle().
					Width(14).
					// Foreground(lipgloss.Color("#FAFAFA")).
					// Foreground(lipgloss.Color("#995399")).
					Bold(true).
					Render(commands[key].Name()))

				sb.WriteString(lipgloss.NewStyle().
					Align(lipgloss.Left).
					Width(TextWidth - 16).
					Foreground(lipgloss.AdaptiveColor{Light: "236", Dark: "248"}).
					Italic(true).
					Render(` ` + commands[key].Short))
			}

			fmt.Print(lipgloss.NewStyle().
				Border(lipgloss.RoundedBorder(), true, true).
				// BorderForeground(lipgloss.Color("#995399")). //d06fcf
				Padding(0, 1).
				Width(TextWidth - 10).
				// Bold(true).
				Render(sb.String()))

			fmt.Println()
			fmt.Println()

			fmt.Println(lipgloss.NewStyle().
				Foreground(lipgloss.AdaptiveColor{Light: "236", Dark: "248"}).
				Render(" For more info on a specific command, use:"))
			fmt.Println("  > " + lipgloss.NewStyle().Italic(true).Render("fig help [command]"))

			fmt.Println()

		} else {
			// Help page for subcommand

			fmt.Println(lipgloss.
				NewStyle().
				Border(lipgloss.NormalBorder(), true, false).
				Padding(0, 1).
				Width(TextWidth).
				Render(lipgloss.NewStyle().Bold(true).Render(cmd.Name()) + " - " + cmd.Short))

			fmt.Println(lipgloss.NewStyle().
				Width(TextWidth).
				Align(lipgloss.Left).
				Bold(true).
				Render("Usage"))

			fmt.Println(lipgloss.NewStyle().
				Padding(0, 2).
				Width(TextWidth).
				Align(lipgloss.Left).
				Render(cmd.UseLine()))

			if cmd.HasSubCommands() {
				fmt.Println(lipgloss.NewStyle().
					Padding(0, 2).
					Width(TextWidth).
					Align(lipgloss.Left).
					Render(cmd.CommandPath() + " [subcommand]"))
			}

			if len(cmd.ValidArgs) > 0 {
				fmt.Println(lipgloss.NewStyle().
					MarginTop(1).
					Width(TextWidth).
					Align(lipgloss.Left).
					Bold(true).
					Render("Argument Options"))

				fmt.Println(lipgloss.NewStyle().
					Padding(0, 2).
					Width(TextWidth).
					Align(lipgloss.Left).
					Render(strings.Join(cmd.ValidArgs, " ")))

			}

			if cmd.Annotations["figcli.command.argDescriptions"] != "" {
				fmt.Println(lipgloss.NewStyle().
					MarginTop(1).
					Width(TextWidth).
					Align(lipgloss.Left).
					Bold(true).
					Render("Arguments"))

				fmt.Println(lipgloss.NewStyle().
					Padding(0, 2).
					Width(TextWidth).
					Align(lipgloss.Left).
					Render(cmd.Annotations["figcli.command.argDescriptions"]))
			}

			if cmd.HasSubCommands() {
				fmt.Println(lipgloss.NewStyle().
					MarginTop(1).
					Width(TextWidth).
					Align(lipgloss.Left).
					Bold(true).
					Render("Subcommands"))

				for _, c := range cmd.Commands() {
					if !c.Hidden {
						fmt.Print(" > ")
						fmt.Print(lipgloss.NewStyle().
							Width(12).
							Bold(true).
							Align(lipgloss.Left).
							Render(c.Name()))

						fmt.Println(lipgloss.NewStyle().
							Padding(0, 2).
							Width(TextWidth).
							Align(lipgloss.Left).
							Italic(true).
							Render(c.Short))
					}
				}
			}

			if cmd.HasFlags() {
				fmt.Println(lipgloss.NewStyle().
					MarginTop(1).
					Width(TextWidth).
					Align(lipgloss.Left).
					Bold(true).
					Render("Flags"))

				cmd.Flags().VisitAll(func(f *pflag.Flag) {
					fmt.Print(lipgloss.NewStyle().
						PaddingLeft(1).
						Width(15).
						Align(lipgloss.Left).
						Render("--" + f.Name))

					fmt.Println(lipgloss.NewStyle().
						Padding(0, 2).
						Width(TextWidth - 15).
						Align(lipgloss.Left).
						Italic(true).
						Render(f.Usage))
				})
			}

			if cmd.HasSubCommands() {
				fmt.Println()
				fmt.Println("For more help on a specific command, use:")
				fmt.Println(" > " + lipgloss.NewStyle().Italic(true).Render(cmd.CommandPath()+" [subcommand] --help"))
			}

			fmt.Println()
		}
	})

	rootCmd.AddCommand(alpha.NewCmdAlpha())
	rootCmd.AddCommand(app.NewCmdApp())
	rootCmd.AddCommand(careers.NewCmdCareers())
	rootCmd.AddCommand(community.NewCmdCommunity())
	rootCmd.AddCommand(contrib.NewCmdContrib())
	rootCmd.AddCommand(debug.NewCmdDebug())
	rootCmd.AddCommand(dev.NewCmdDev())
	rootCmd.AddCommand(docs.NewCmdDocs())
	rootCmd.AddCommand(doctor.NewCmdDoctor())
	rootCmd.AddCommand(hook.NewCmdHook())
	rootCmd.AddCommand(integrations.NewCmdIntegrations())
	rootCmd.AddCommand(invite.NewCmdInvite())
	rootCmd.AddCommand(issue.NewCmdIssue())
	rootCmd.AddCommand(launch.NewCmdLaunch())
	rootCmd.AddCommand(quit.NewCmdQuit())
	rootCmd.AddCommand(restart.NewCmdRestart())
	rootCmd.AddCommand(settings.NewCmdSettings())
	rootCmd.AddCommand(source.NewCmdSource())
	rootCmd.AddCommand(specs.NewCmdSpecs())
	rootCmd.AddCommand(theme.NewCmdTheme())
	rootCmd.AddCommand(tips.NewCmdTips())
	rootCmd.AddCommand(tweet.NewCmdTweet())
	rootCmd.AddCommand(update.NewCmdUpdate())
	rootCmd.AddCommand(user.NewCmdUser())

	rootCmd.AddCommand(diagnostic.NewCmdDiagnostic())
	rootCmd.AddCommand(logout.NewCmdLogout())
	rootCmd.AddCommand(onboarding.NewCmdOnboarding())

	rootCmd.AddCommand(genFigSpec.NewCmdGenFigSpec())

	// DO NOT REMOVE UNTIL HYPER EXTENSION USES `fig hook keyboard-focus-changed` format
	rootCmd.AddCommand(legacy_focus_changed.NewCmdKeyboardFocusChanged(true))

	if err := rootCmd.Execute(); err != nil {
		logging.Log("root error:", err.Error())
		os.Exit(1)
	}
}
