package cmd

import (
	"fig-cli/diagnostics"
	fig_ipc "fig-cli/fig-ipc"
	fig_proto "fig-cli/fig-proto"
	"fmt"
	"os"
	"strings"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

const TextWidth = 60

var rootCmd = &cobra.Command{
	Use:   "fig",
	Short: "CLI to interact with Fig",
	Annotations: map[string]string{
		"fig.command": "true",
	},
	// This is stupid, I hate golang, why can I not decompose multiple returns into a single line?
	Version: func() string {
		version, _ := diagnostics.GetFigVersion()
		return version
	}(),
	Run: func(cmd *cobra.Command, args []string) {
		response, err := fig_ipc.RunOpenUiElementCommand(fig_proto.UiElement_MENU_BAR)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		if response != "" {
			fmt.Printf("\n%s\n\n", response)
		}
	},
}

func Execute() {
	rootCmd.SetHelpFunc(func(cmd *cobra.Command, args []string) {
		commandGroups := make(map[string][]*cobra.Command)

		if !cmd.HasParent() {
			println(lipgloss.NewStyle().
				Bold(true).
				Padding(1, 0).
				Width(TextWidth).
				Align(lipgloss.Center).
				Render(`/@@@@@@@@@@@\    /@@@@@@@ @@         
@@@@@@/    \@    @@           /@@@@@@\
@@@@@@      @    @@       /@ /@@    @@
@@@@@@      @    @@@@@@@  @@ @@    @@@
@@@@@@\    /@    @@       @@  \@@@@/@@
\@@@@@@@@@@@/    @/       @/        @@
                              \@@__@@/`))

			fmt.Println(lipgloss.
				NewStyle().
				PaddingBottom(1).
				Width(TextWidth).
				Align(lipgloss.Center).
				Render(
					"",
				))

			for _, c := range cmd.Commands() {
				if c.Annotations["figcli.command.categories"] != "" {
					commandGroups[c.Annotations["figcli.command.categories"]] =
						append(commandGroups[c.Annotations["figcli.command.categories"]], c)
				}
			}

			for _, command := range commandGroups {

				fmt.Println(lipgloss.
					NewStyle().
					Border(lipgloss.NormalBorder(), true, false).
					Padding(0, 1).
					Width(TextWidth).
					Bold(true).
					Render("Common Subcommands"))

				fmt.Println(lipgloss.
					NewStyle().
					PaddingBottom(1).
					PaddingLeft(1).
					Width(TextWidth).
					Render(
						lipgloss.NewStyle().
							Bold(true).
							Render(`Usage: `) +
							lipgloss.NewStyle().
								Italic(true).
								Render(`fig [subcommand]`),
					))

				for _, c := range command {
					fmt.Print(" > ")
					fmt.Print(lipgloss.NewStyle().
						Width(14).
						Bold(true).
						Render(c.Name()))

					fmt.Println(lipgloss.NewStyle().
						Align(lipgloss.Left).
						Width(TextWidth - 16).
						Italic(true).
						Render(` ` + c.Short))
				}

				fmt.Println()

				fmt.Println("For more help on a specific command, use:")
				fmt.Println(" > " + lipgloss.NewStyle().Italic(true).Render("fig help [command]"))

				fmt.Println()
			}
		} else {
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

	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
