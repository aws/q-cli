package wrapped

import (
	"fmt"
	"os"
	"os/exec"
	"os/user"
	"path"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
	"golang.org/x/term"
)

type History struct {
	Command   string
	ExitCode  int
	Shell     string
	SessionId string
	Cwd       string
	Time      int
}

func loadHistory() ([]History, error) {
	// Load history file
	user, err := user.Current()
	if err != nil {
		return nil, err
	}

	historyPath := path.Join(user.HomeDir, ".fig", "history")

	historyData, err := os.ReadFile(historyPath)
	if err != nil {
		return nil, err
	}
	historyString := string(historyData)

	// Parse History data
	history := []History{}

	historyRe := regexp.MustCompile("- command: (.*)\n  exit_code: (.*)\n  shell: (.*)\n  session_id: (.*)\n  cwd: (.*)\n  time: (.*)")
	historyMatches := historyRe.FindAllStringSubmatch(historyString, -1)

	for _, historyMatch := range historyMatches {
		exitCode, _ := strconv.Atoi(historyMatch[2])
		time, _ := strconv.Atoi(historyMatch[6])

		history = append(history, History{
			Command:   historyMatch[1],
			ExitCode:  exitCode,
			Shell:     historyMatch[3],
			SessionId: historyMatch[4],
			Cwd:       historyMatch[5],
			Time:      time,
		})
	}

	return history, nil
}

type CommandUsage struct {
	Command string
	Count   int
}

type WorkingDirUsage struct {
	WorkingDir string
	Count      int
}

type ShellUsage struct {
	Shell string
	Count int
}

type AliasUsage struct {
	Alias string
	Count int
}

type HistoryMetrics struct {
	TopCommandsUsage       []CommandUsage
	TopWorkingDirs         []WorkingDirUsage
	TopShells              []ShellUsage
	TopAliases             []AliasUsage
	LongestPipedSequence   string
	MostCommonTimeOfDay    map[int]int
	MostCommonDayOfWeek    map[time.Weekday]int
	CharactersSavedByAlias int
}

func getAlias(shell string) (map[string]string, error) {
	if shell == "bash" || shell == "zsh" || shell == "fish" {
		commandOutput, _ := exec.Command(shell, "-ic", "alias").Output()
		aliases := make(map[string]string)

		for _, alias := range strings.Split(string(commandOutput), "\n") {
			if alias == "" {
				continue
			}

			if strings.HasPrefix(alias, "alias ") {
				alias = strings.TrimSpace(alias[5:])
			}

			splitChar := ""
			if shell == "fish" {
				splitChar = " "
			} else {
				splitChar = "="
			}

			aliasSplit := strings.SplitN(alias, splitChar, 2)
			if len(aliasSplit) != 2 {
				continue
			}

			key := strings.TrimSpace(aliasSplit[0])
			command := strings.TrimSpace(aliasSplit[1])

			if shell == "bash" || shell == "zsh" {
				command = strings.TrimLeft(command, "'")
				command = strings.TrimRight(command, "'")
			}

			aliases[key] = command
		}

		return aliases, nil
	} else {
		return nil, fmt.Errorf("shell %v not supported", shell)
	}
}

func Metrics(history []History) HistoryMetrics {
	metrics := HistoryMetrics{}

	commandsUsageMap := map[string]int{}
	workingDirMap := map[string]int{}
	shellMap := map[string]int{}
	aliasMap := map[string]int{}

	longestPipedSequence := ""
	pipesInSequence := 0

	timeOfDay := map[int]int{}
	dayOfWeek := map[time.Weekday]int{}

	shellAliases := make(map[string]map[string]string)

	zshAliases, _ := getAlias("zsh")
	bashAliases, _ := getAlias("bash")
	fishAliases, _ := getAlias("fish")

	shellAliases["zsh"] = zshAliases
	shellAliases["bash"] = bashAliases
	shellAliases["fish"] = fishAliases

	charsSavedByAlias := 0

	for _, h := range history {
		workingDirMap[h.Cwd]++

		command := strings.SplitN(h.Command, " ", 2)[0]
		if command != "" && command != "\\n" {
			if shellAliases[h.Shell] != nil && shellAliases[h.Shell][command] != "" {
				deAliasedCommmand := shellAliases[h.Shell][command]
				deAliasedCommmand = strings.SplitN(deAliasedCommmand, " ", 2)[0]
				commandsUsageMap[deAliasedCommmand]++
			} else {
				commandsUsageMap[command]++
			}
		}

		if shellAliases[h.Shell] != nil && shellAliases[h.Shell][command] != "" {
			charsSavedByAlias += len(shellAliases[h.Shell][command]) - len(command)
			aliasMap[command]++
		}

		pipeCount := strings.Count(h.Command, "|")
		if pipeCount > pipesInSequence {
			longestPipedSequence = h.Command
			pipesInSequence = pipeCount
		}

		shellMap[h.Shell]++

		commandTime := time.Unix(int64(h.Time), 0).Local()
		timeOfDay[commandTime.Hour()]++
		dayOfWeek[commandTime.Weekday()]++
	}

	metrics.LongestPipedSequence = longestPipedSequence

	metrics.MostCommonTimeOfDay = timeOfDay
	metrics.MostCommonDayOfWeek = dayOfWeek

	metrics.CharactersSavedByAlias = charsSavedByAlias

	// Convert commandsUsageMap to list of CommandUsage
	for command, count := range commandsUsageMap {
		metrics.TopCommandsUsage = append(metrics.TopCommandsUsage, CommandUsage{
			Command: command,
			Count:   count,
		})
	}

	// Sort CommandUsage by count
	sort.Slice(metrics.TopCommandsUsage, func(i, j int) bool {
		return metrics.TopCommandsUsage[i].Count > metrics.TopCommandsUsage[j].Count
	})

	// Convert workingDirMap to list of WorkingDirUsage
	for workingDir, count := range workingDirMap {
		metrics.TopWorkingDirs = append(metrics.TopWorkingDirs, WorkingDirUsage{
			WorkingDir: workingDir,
			Count:      count,
		})
	}

	// Sort WorkingDirUsage by count
	sort.Slice(metrics.TopWorkingDirs, func(i, j int) bool {
		return metrics.TopWorkingDirs[i].Count > metrics.TopWorkingDirs[j].Count
	})

	// Convert shellMap to list of ShellUsage
	for shell, count := range shellMap {
		metrics.TopShells = append(metrics.TopShells, ShellUsage{
			Shell: shell,
			Count: count,
		})
	}

	// Sort ShellUsage by count
	sort.Slice(metrics.TopShells, func(i, j int) bool {
		return metrics.TopShells[i].Count > metrics.TopShells[j].Count
	})

	// Convert aliasMap to list of AliasUsage
	for alias, count := range aliasMap {
		metrics.TopAliases = append(metrics.TopAliases, AliasUsage{
			Alias: alias,
			Count: count,
		})
	}

	// Sort AliasUsage by count
	sort.Slice(metrics.TopAliases, func(i, j int) bool {
		return metrics.TopAliases[i].Count > metrics.TopAliases[j].Count
	})

	return metrics
}

type model struct {
	metrics HistoryMetrics
	page    int
}

func initialModel() model {
	history, _ := loadHistory()
	metrics := Metrics(history)

	return model{
		metrics: metrics,
		page:    0,
	}
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit
		case "enter", " ":
			m.page++
		}
	}

	if m.page >= 8 {
		return m, tea.Quit
	}

	return m, nil
}

func (m model) View() string {
	physicalWidth, physicalHeight, _ := term.GetSize(int(os.Stdout.Fd()))
	doc := strings.Builder{}

	switch m.page {
	// Into page
	case 0:

		fig_logo := `███████╗██╗ ██████╗
██╔════╝██║██╔════╝
█████╗  ██║██║  ███╗
██╔══╝  ██║██║   ██║
██║     ██║╚██████╔╝
╚═╝     ╚═╝ ╚═════╝  Wrapped`
		title := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render(fig_logo)
		caption := lipgloss.NewStyle().Italic(true).Foreground(lipgloss.Color("5")).Render("Here is your 2021 in the shell wrapped up")

		doc.WriteString(lipgloss.JoinVertical(lipgloss.Center, title, caption))

	// Command usage
	case 1:
		maxWorkingDirs := 5
		if len(m.metrics.TopWorkingDirs) < maxWorkingDirs {
			maxWorkingDirs = len(m.metrics.TopWorkingDirs)
		}

		workingDirBackground := lipgloss.Color("5")

		workingDirTitle := lipgloss.NewStyle().Background(workingDirBackground).MarginBottom(1).Bold(true).Render("Top working dirs")

		counts := []string{}
		dirs := []string{}
		for _, workingDir := range m.metrics.TopWorkingDirs[0:maxWorkingDirs] {
			// Pretty print working dir
			user, _ := user.Current()

			workingDirPretty := strings.Replace(workingDir.WorkingDir, user.HomeDir, "~", 1)
			if workingDirPretty[len(workingDirPretty)-1] != '/' {
				workingDirPretty += "/"
			}

			counts = append(counts, fmt.Sprintf("%v", workingDir.Count))
			dirs = append(dirs, workingDirPretty)
		}

		countsStr := lipgloss.JoinVertical(lipgloss.Right, counts...)
		dirsStr := lipgloss.JoinVertical(lipgloss.Left, dirs...)

		dirCountStr := lipgloss.NewStyle().
			Render(lipgloss.JoinHorizontal(lipgloss.Top, countsStr, " ", dirsStr))

		workingDirPanel := lipgloss.NewStyle().
			Padding(1, 2).
			Margin(0, 2, 1, 1).
			Background(workingDirBackground).
			Render(workingDirTitle + "\n" + dirCountStr)

		maxAlias := 5
		if len(m.metrics.TopAliases) < maxAlias {
			maxAlias = len(m.metrics.TopAliases)
		}

		aliasBackground := lipgloss.Color("8")

		aliasTitle := lipgloss.NewStyle().Background(aliasBackground).MarginBottom(1).Bold(true).Render("Top aliases")

		counts = []string{}
		aliases := []string{}
		for _, alias := range m.metrics.TopAliases[0:maxAlias] {
			counts = append(counts, fmt.Sprintf("%v", alias.Count))
			aliases = append(aliases, alias.Alias)
		}

		countsStr = lipgloss.JoinVertical(lipgloss.Right, counts...)
		aliasesStr := lipgloss.JoinVertical(lipgloss.Left, aliases...)

		alisesCountStr := lipgloss.NewStyle().
			Render(lipgloss.JoinHorizontal(lipgloss.Top, countsStr, " ", aliasesStr))

		alisesCountPanel := lipgloss.NewStyle().
			Padding(1, 2).
			Margin(0, 2, 1, 1).
			Background(aliasBackground).
			Render(aliasTitle + "\n" + alisesCountStr)

		maxCommands := 15
		if len(m.metrics.TopCommandsUsage) < maxCommands {
			maxCommands = len(m.metrics.TopCommandsUsage)
		}

		commandsPanelBackground := lipgloss.Color("2")

		commandPageTitle := lipgloss.NewStyle().Bold(true).Background(commandsPanelBackground).PaddingBottom(1).Render("Top commands")

		counts = []string{}
		commands := []string{}
		for _, command := range m.metrics.TopCommandsUsage[0:maxCommands] {
			counts = append(counts, fmt.Sprintf("%v", command.Count))
			commands = append(commands, command.Command)
		}

		countsStr = lipgloss.JoinVertical(lipgloss.Right, counts...)
		commandsStr := lipgloss.JoinVertical(lipgloss.Left, commands...)

		commmandsStr := lipgloss.NewStyle().
			Render(lipgloss.JoinHorizontal(lipgloss.Top, countsStr, " ", commandsStr))

		commandPanel := lipgloss.NewStyle().
			Padding(1, 2).
			Margin(0, 0, 1, 1).
			Background(commandsPanelBackground).
			Render(lipgloss.JoinVertical(lipgloss.Left, commandPageTitle, commmandsStr))

		dayOfWeekHistogramTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Weekly Activity")

		maxCount := 0
		for _, count := range m.metrics.MostCommonDayOfWeek {
			if count > maxCount {
				maxCount = count
			}
		}

		daysOfWeek := []string{"Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"}
		counts = []string{}
		for i := 0; i < 7; i++ {
			counts = append(counts, strings.Repeat("█", int(float64(m.metrics.MostCommonDayOfWeek[time.Weekday(i)])/float64(maxCount)*20)))
		}

		daysOfWeekStr := lipgloss.JoinVertical(lipgloss.Right, daysOfWeek...)
		countsStr = lipgloss.JoinVertical(lipgloss.Left, counts...)

		daysOfWeekHistogramStr := lipgloss.NewStyle().
			Render(lipgloss.JoinHorizontal(lipgloss.Top, daysOfWeekStr, " ", countsStr))

		dayOfWeekHistogramPanel := lipgloss.NewStyle().
			Background(lipgloss.Color("6")).
			Padding(1, 2).
			Render(dayOfWeekHistogramTitle + "\n" + daysOfWeekHistogramStr)

		doc.WriteString(
			lipgloss.JoinVertical(
				lipgloss.Center,
				lipgloss.NewStyle().Bold(true).MarginBottom(1).Render("Fig Wrapped"),
				lipgloss.JoinHorizontal(lipgloss.Center, commandPanel, " ",
					lipgloss.JoinVertical(lipgloss.Center, workingDirPanel, alisesCountPanel)),
				lipgloss.JoinHorizontal(lipgloss.Center, dayOfWeekHistogramPanel, "  ", dayOfWeekHistogramPanel)))

	// Working dirs
	case 2:

	// Longest piped sequence
	case 3:
		longestPipedSequencePageTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Longest piped sequence")
		longestPipedSequenceStr := lipgloss.NewStyle().Align(lipgloss.Left).Render(m.metrics.LongestPipedSequence)
		longestPipedSequencePage := lipgloss.JoinVertical(lipgloss.Center, longestPipedSequencePageTitle, longestPipedSequenceStr)
		doc.WriteString(longestPipedSequencePage)

	// Time of Day Histogram
	case 4:
		timeOfDayHistogramPageTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Time of Day Histogram")

		maxCount := 0
		for _, count := range m.metrics.MostCommonTimeOfDay {
			if count > maxCount {
				maxCount = count
			}
		}

		timeOfDayHistogramStrBuilder := strings.Builder{}
		for i := 0; i < 24; i++ {
			time := ""
			if i == 0 {
				time = fmt.Sprintf("%3d AM", 12)
			} else if i < 12 {
				time = fmt.Sprintf("%3d AM", i)
			} else if i == 12 {
				time = fmt.Sprintf("%3d PM", i)
			} else {
				time = fmt.Sprintf("%3d PM", i-12)
			}

			timeOfDayHistogramStrBuilder.WriteString(
				fmt.Sprintf("%v %v\n",
					time,
					strings.Repeat("█", int(float64(m.metrics.MostCommonTimeOfDay[i])/float64(maxCount)*70))))
		}

		timeOfDayHistogramStr := lipgloss.NewStyle().Align(lipgloss.Left).Render(timeOfDayHistogramStrBuilder.String())

		timeOfDayHistogramPage := lipgloss.JoinVertical(lipgloss.Center, timeOfDayHistogramPageTitle, timeOfDayHistogramStr)

		doc.WriteString(timeOfDayHistogramPage)

	// Day of Week Histogram
	case 5:

	// Top Shells
	case 6:
		topShellPageTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Top Shells")

		maxShells := 3
		if len(m.metrics.TopShells) < maxShells {
			maxShells = len(m.metrics.TopShells)
		}

		shellsStrBuilder := strings.Builder{}
		for _, shell := range m.metrics.TopShells[0:maxShells] {
			if shell.Count > 0 {
				shellsStrBuilder.WriteString(fmt.Sprintf("%5v: %v\n", shell.Count, shell.Shell))
			}
		}

		shellsStr := lipgloss.NewStyle().Align(lipgloss.Left).Render(shellsStrBuilder.String())

		topShellPage := lipgloss.JoinVertical(lipgloss.Center, topShellPageTitle, shellsStr)

		doc.WriteString(topShellPage)

	// Shell Aliases
	case 7:
		shellAliasesPageTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Keystrokes saved by using aliases")

		shellAliasesPage := fmt.Sprintf("%v", m.metrics.CharactersSavedByAlias)

		doc.WriteString(lipgloss.JoinVertical(lipgloss.Center, shellAliasesPageTitle, shellAliasesPage))

	// End Screen
	case 8:
		doc.WriteString("Thanks for using Fig in 2021!")
	}

	fullPage := ""

	year := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Background(lipgloss.Color("1")).Bold(true).Render("2021")
	inReview := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Background(lipgloss.Color("2")).Bold(true).Render("In Review")
	commands := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Background(lipgloss.Color("3")).Bold(true).Render("Commands")
	atFig := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Background(lipgloss.Color("4")).Bold(true).Render("@fig")

	statusBarLeft := year + inReview
	statusBarRight := lipgloss.NewStyle().Background(lipgloss.Color("5")).Width(physicalWidth - lipgloss.Width(statusBarLeft)).Align(lipgloss.Right).Render(commands + atFig)

	statusBar := statusBarLeft + statusBarRight

	if m.page < 8 && m.page != 1 {
		nextPage := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("8")).MarginTop(2).Render("[Press Enter to continue]")
		fullPage = lipgloss.JoinVertical(lipgloss.Center, doc.String(), nextPage)
	} else {
		fullPage = doc.String()
	}

	page := lipgloss.Place(
		physicalWidth,
		physicalHeight-1,
		lipgloss.Center,
		lipgloss.Center,
		fullPage)

	return lipgloss.JoinVertical(lipgloss.Center, page, statusBar)
}

func NewCmdWrapped() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "wrapped",
		Short:  "How did you use the shell in 2021",
		Long:   "How did you use the shell in 2021",
		Hidden: true,
		Run: func(cmd *cobra.Command, arg []string) {
			p := tea.NewProgram(initialModel())
			if err := p.Start(); err != nil {
				fmt.Printf("Alas, there's been an error: %v", err)
				os.Exit(1)
			}
		},
	}

	return cmd
}
