package wrapped

import (
	"fmt"
	"math"
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

const (
	timeGroups = 24
)

func truncate(command string, maxlength int) string {
	length := len(command)
	if length > maxlength {
		command = command[:maxlength-3] + "..."
	}
	return command
}

func truncatePath(path string, maxlength int) string {
	length := len(path)

	for length > maxlength {
		path = strings.TrimLeft(path, "./")
		split := strings.SplitN(path, "/", 2)
		if len(split) == 2 {
			path = "./" + split[1]
			length = len(path)
		} else {
			path = truncate(path, maxlength)
			break
		}
	}

	return path
}

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
	TimeOfDay              map[int]int
	DayOfWeek              map[time.Weekday]int
	CharactersSavedByAlias int
	TotalCommands          int
	Keystrokes             int
	Commits                int
	ShortesGitCommit       string
	ShortedCommitTime      int
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

	gitCommitRegex := regexp.MustCompile(`-m (\\\"(.*)\\\")|('(.*)')`)
	shortestGitCommitLen := math.MaxInt

	for _, h := range history {
		workingDirMap[h.Cwd]++

		metrics.Keystrokes += len(h.Command)

		if strings.HasPrefix(h.Command, "git commit") {
			metrics.Commits++
			gitCommitRegexMatch := gitCommitRegex.FindStringSubmatch(h.Command)
			if len(gitCommitRegexMatch) > 0 {
				gitCommit := ""

				if gitCommitRegexMatch[2] != "" {
					gitCommit = gitCommitRegexMatch[2]
				} else if gitCommitRegexMatch[4] != "" {
					gitCommit = gitCommitRegexMatch[4]
				}

				if len(gitCommit) > 1 && len(gitCommit) < shortestGitCommitLen {
					shortestGitCommitLen = len(gitCommit)
					metrics.ShortesGitCommit = gitCommit
					metrics.ShortedCommitTime = h.Time
				}
			}
		}

		command := strings.SplitN(h.Command, " ", 2)[0]
		if command != "" && command != "\\n" {
			if shellAliases[h.Shell] != nil && shellAliases[h.Shell][command] != "" {
				deAliasedCommmand := shellAliases[h.Shell][command]
				deAliasedCommmand = strings.SplitN(deAliasedCommmand, " ", 2)[0]
				commandsUsageMap[deAliasedCommmand]++
			} else {
				commandsUsageMap[command]++
			}

			metrics.TotalCommands++
		}

		if shellAliases[h.Shell] != nil && shellAliases[h.Shell][command] != "" {
			metrics.CharactersSavedByAlias += len(shellAliases[h.Shell][command]) - len(command)
			aliasMap[command]++
		}

		pipeCount := strings.Count(h.Command, "|")
		if pipeCount > pipesInSequence {
			metrics.LongestPipedSequence = h.Command
			pipesInSequence = pipeCount
		}

		shellMap[h.Shell]++

		commandTime := time.Unix(int64(h.Time), 0).Local()

		groups := timeGroups
		minInDay := 24 * 60
		group := int(float32(commandTime.Hour()*60+commandTime.Minute()) / float32(minInDay) * float32(groups))
		timeOfDay[group]++

		dayOfWeek[commandTime.Weekday()]++
	}

	metrics.TimeOfDay = timeOfDay
	metrics.DayOfWeek = dayOfWeek

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
	history, err := loadHistory()
	if err != nil {
		fmt.Printf("\n→ You haven't had Fig installed long enough to generate your year in review.\n  Try again in a couple of days!\n\n")
		os.Exit(0)
	}

	metrics := Metrics(history)

	if len(history) < 100 {
		fmt.Printf("\n→ You haven't had Fig installed long enough to generate your year in review.\n  Try again in a couple of days!\n\n")
		os.Exit(0)
	}

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

	if m.page >= 2 {
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

		asciiArt := `.--~~~~~~~~~~~~~------.
/--===============------\
| |⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺|     |
| | ` + lipgloss.NewStyle().Blink(true).Render(">") + `             |     |
| |               |     |
| |               |     |
| |_______________|     |
|                   ::::|
'======================='
//-'-'-'-'-'-'-'-'-'-'-\\
//_'_'_'_'_'_'_'_'_'_'_'_\\
[-------------------------]
\_________________________/


`

		// asciiArt := ` .--~~~~~~~~~~~~~------.
		// /--===============------\
		// | |⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺⎺|     |
		// | |    ███▛▀▀▜    |     |
		// | |    ███▌  ▐    |     |
		// | |    ███▙▄▄▟    |     |
		// | |_______________|     |
		// |                   ::::|
		// '======================='
		// //-'-'-'-'-'-'-'-'-'-'-\\
		// //_'_'_'_'_'_'_'_'_'_'_'_\\
		// [-------------------------]
		// \_________________________/

		// `

		doc.WriteString(asciiArt)

		// 		fig_logo := `███████╗██╗ ██████╗
		// ██╔════╝██║██╔════╝
		// █████╗  ██║██║  ███╗
		// ██╔══╝  ██║██║   ██║
		// ██║     ██║╚██████╔╝
		// ╚═╝     ╚═╝ ╚═════╝  Wrapped`
		// 		title := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render(fig_logo)

		thanks := lipgloss.NewStyle().Bold(true).Render("Thanks so much for using Fig in 2021!")

		// caption := lipgloss.NewStyle().Italic(true).Foreground(lipgloss.Color("5")).Render("Here is your 2021 in the shell wrapped up")

		doc.WriteString(lipgloss.JoinVertical(lipgloss.Center, thanks))

	// Command usage
	case 1, 2:

		figAscii := `███████╗██╗ ██████╗
██╔════╝██║██╔════╝
█████╗  ██║██║  ███╗
██╔══╝  ██║██║   ██║
██║     ██║╚██████╔╝ 2021
╚═╝     ╚═╝ ╚═════╝  Wrapped`

		logoBox := lipgloss.NewStyle().
			Border(lipgloss.ThickBorder()).
			BorderForeground(lipgloss.Color("2")).
			Padding(1, 2).
			Bold(true).
			Render(figAscii)

		maxWorkingDirs := 5
		if len(m.metrics.TopWorkingDirs) < maxWorkingDirs {
			maxWorkingDirs = len(m.metrics.TopWorkingDirs)
		}

		workingDirTitle := lipgloss.NewStyle().MarginBottom(1).Bold(true).Render("Most Used Directories")

		counts := []string{}
		dirs := []string{}
		for _, workingDir := range m.metrics.TopWorkingDirs[0:maxWorkingDirs] {
			// Pretty print working dir
			user, _ := user.Current()

			workingDirPretty := strings.Replace(workingDir.WorkingDir, user.HomeDir, "~", 1)
			if workingDirPretty[len(workingDirPretty)-1] != '/' {
				workingDirPretty += "/"
			}

			workingDirPretty = truncatePath(workingDirPretty, 25)

			counts = append(counts, fmt.Sprintf("%v", workingDir.Count))
			dirs = append(dirs, workingDirPretty)
		}

		countsStr := lipgloss.JoinVertical(lipgloss.Right, counts...)
		dirsStr := lipgloss.JoinVertical(lipgloss.Left, dirs...)

		dirCountStr := lipgloss.NewStyle().
			Render(lipgloss.JoinHorizontal(lipgloss.Top, countsStr, " ", dirsStr))

		workingDirPanel := lipgloss.NewStyle().
			Padding(1, 2).
			Border(lipgloss.DoubleBorder()).
			BorderForeground(lipgloss.Color("3")).
			Width(lipgloss.Width(logoBox) - 2).
			Render(workingDirTitle + "\n" + dirCountStr)

		// maxAlias := 4
		// if len(m.metrics.TopAliases) < maxAlias {
		// 	maxAlias = len(m.metrics.TopAliases)
		// }

		// aliasTitle := lipgloss.NewStyle().MarginBottom(1).Bold(true).Render("Top aliases")

		// counts = []string{}
		// aliases := []string{}
		// for _, alias := range m.metrics.TopAliases[0:maxAlias] {
		// 	counts = append(counts, fmt.Sprintf("%v", alias.Count))
		// 	aliases = append(aliases, truncateCommand(alias.Alias, 25))
		// }

		// countsStr = lipgloss.JoinVertical(lipgloss.Right, counts...)
		// aliasesStr := lipgloss.JoinVertical(lipgloss.Left, aliases...)

		// alisesCountStr := lipgloss.NewStyle().
		// 	Render(lipgloss.JoinHorizontal(lipgloss.Top, countsStr, " ", aliasesStr))

		// alisesCountPanel := lipgloss.NewStyle().
		// 	Padding(1, 2).
		// 	Border(lipgloss.RoundedBorder()).
		// 	Render(aliasTitle + "\n" + alisesCountStr)

		//			Width(lipgloss.Width(workingDirPanel) - 2).

		maxCommands := 15
		if len(m.metrics.TopCommandsUsage) < maxCommands {
			maxCommands = len(m.metrics.TopCommandsUsage)
		}

		commandPageTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Top Commands")

		counts = []string{}
		commands := []string{}
		for _, command := range m.metrics.TopCommandsUsage[0:maxCommands] {
			counts = append(counts, fmt.Sprintf("%v", command.Count))
			commands = append(commands, truncate(command.Command, 15))
		}

		countsStr = lipgloss.JoinVertical(lipgloss.Right, counts...)
		commandsStr := lipgloss.JoinVertical(lipgloss.Left, commands...)

		commmandsStr := lipgloss.NewStyle().
			Render(lipgloss.JoinHorizontal(lipgloss.Top, countsStr, " ", commandsStr))

		commandPanel := lipgloss.NewStyle().
			Padding(1, 2).
			Border(lipgloss.NormalBorder()).
			BorderForeground(lipgloss.Color("4")).
			Width(25).
			Render(lipgloss.JoinVertical(lipgloss.Left, commandPageTitle, commmandsStr))

		dayOfWeekHistogramTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Weekly Activity")

		maxCount := 0
		for _, count := range m.metrics.DayOfWeek {
			if count > maxCount {
				maxCount = count
			}
		}

		daysOfWeek := []string{"Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"}
		counts = []string{}
		for i := 0; i < 7; i++ {
			counts = append(counts, strings.Repeat("█", int(float64(m.metrics.DayOfWeek[time.Weekday(i)])/float64(maxCount)*20)))
		}

		daysOfWeekStr := lipgloss.JoinVertical(lipgloss.Right, daysOfWeek...)
		countsStr = lipgloss.JoinVertical(lipgloss.Left, counts...)

		daysOfWeekHistogramStr := lipgloss.NewStyle().
			Render(lipgloss.JoinHorizontal(lipgloss.Top, daysOfWeekStr, " ", countsStr))

		dayOfWeekHistogramPanel := lipgloss.NewStyle().
			Padding(1, 2).
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("5")).
			Render(dayOfWeekHistogramTitle + "\n" + daysOfWeekHistogramStr)

		timeOfDayHistogramPageTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Daily Activity")

		maxCount = 0
		for _, count := range m.metrics.TimeOfDay {
			if count > maxCount {
				maxCount = count
			}
		}

		width := 20
		timeOfDayHistogramBars := []string{}
		for i := -0.5; i < timeGroups/2; i++ {
			i_1 := int(i * 2)
			i_2 := int(i*2 + 1)
			scaledTime1 := int(float64(m.metrics.TimeOfDay[i_1]) / float64(maxCount) * float64(width))
			scaledTime2 := int(float64(m.metrics.TimeOfDay[i_2]) / float64(maxCount) * float64(width))

			histogramBuilder := strings.Builder{}
			for j := 0; j < width; j++ {
				if scaledTime1 > j && scaledTime2 > j {
					histogramBuilder.WriteString("█")
				} else if scaledTime1 > j && scaledTime2 <= j {
					histogramBuilder.WriteString("▀")
				} else if scaledTime1 <= j && scaledTime2 > j {
					histogramBuilder.WriteString("▄")
				} else if scaledTime1 <= j && scaledTime2 <= j {
					histogramBuilder.WriteString(" ")
				}
			}

			timeOfDayHistogramBars = append(timeOfDayHistogramBars, histogramBuilder.String())
		}

		timeOfDayHistogramBarsStr := lipgloss.JoinVertical(lipgloss.Right, timeOfDayHistogramBars...)

		timeOfDayHistogramStr := lipgloss.NewStyle().
			Align(lipgloss.Left).
			Render(timeOfDayHistogramBarsStr)

		timeOfDayLabels := []string{}
		for i := 0; i < timeGroups/2+1; i++ {
			if i == 0 {
				timeOfDayLabels = append(timeOfDayLabels, "12am ")
			} else if i == timeGroups/2 {
				timeOfDayLabels = append(timeOfDayLabels, "12am ")
			} else if i == timeGroups/4 {
				timeOfDayLabels = append(timeOfDayLabels, "12pm ")
			} else if i == timeGroups/8 {
				timeOfDayLabels = append(timeOfDayLabels, "6am ")
			} else if i == timeGroups/8*3 {
				timeOfDayLabels = append(timeOfDayLabels, "6pm ")
			} else {
				timeOfDayLabels = append(timeOfDayLabels, "")

			}
		}

		timeOfDayLabelsStr := lipgloss.JoinVertical(lipgloss.Right, timeOfDayLabels...)

		timeOfDayStr := lipgloss.JoinHorizontal(lipgloss.Top, timeOfDayLabelsStr, timeOfDayHistogramStr)

		timeOfDayHistogramPage := lipgloss.NewStyle().
			Border(lipgloss.ThickBorder()).
			BorderForeground(lipgloss.Color("6")).
			Padding(1, 2).
			Render(lipgloss.JoinVertical(lipgloss.Left, timeOfDayHistogramPageTitle, timeOfDayStr))

		commitMsgSummary := ""

		if m.metrics.ShortedCommitTime != 0 {
			commitTime := time.Unix(int64(m.metrics.ShortedCommitTime), 0).Local()
			commitTimeStr := commitTime.Format("Jan 2")
			commitMsgSummary = lipgloss.JoinVertical(lipgloss.Left,
				lipgloss.NewStyle().Bold(true).Render("Shortest Commit Message"),
				"'"+truncate(m.metrics.ShortesGitCommit, 12)+"' on "+commitTimeStr)
		} else {
			commitMsgSummary = lipgloss.JoinVertical(lipgloss.Left,
				lipgloss.NewStyle().Bold(true).Render("Shortest Commit Message"),
				"No commits found")
		}

		statsSummary := lipgloss.NewStyle().
			Padding(1, 2).
			Width(lipgloss.Width(dayOfWeekHistogramPanel) - 2).
			Border(lipgloss.DoubleBorder()).
			BorderForeground(lipgloss.Color("9")).
			Render(commitMsgSummary)

		shareText := lipgloss.NewStyle().
			MarginTop(1).
			Render("🎁 Share your " + lipgloss.NewStyle().Bold(true).Render("#FigWrapped") + " with " + lipgloss.NewStyle().Bold(true).Render("@fig"))

		doc.WriteString(
			lipgloss.JoinVertical(
				lipgloss.Center,
				lipgloss.JoinHorizontal(lipgloss.Center, commandPanel,
					lipgloss.JoinVertical(lipgloss.Left, logoBox, workingDirPanel)),
				lipgloss.JoinHorizontal(lipgloss.Center,
					lipgloss.JoinVertical(lipgloss.Center, statsSummary, dayOfWeekHistogramPanel),
					timeOfDayHistogramPage),
				shareText))

	// Working dirs

	// Longest piped sequence
	case 3:
		longestPipedSequencePageTitle := lipgloss.NewStyle().Bold(true).PaddingBottom(1).Render("Longest piped sequence")
		longestPipedSequenceStr := lipgloss.NewStyle().Align(lipgloss.Left).Render(m.metrics.LongestPipedSequence)
		longestPipedSequencePage := lipgloss.JoinVertical(lipgloss.Center, longestPipedSequencePageTitle, longestPipedSequenceStr)
		doc.WriteString(longestPipedSequencePage)

	// Time of Day Histogram
	case 4:

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

	statusBarForeground := lipgloss.Color("15")

	yearColor := lipgloss.Color("0")
	inReviewColor := lipgloss.Color("8")
	commandsColor := lipgloss.Color("7")
	aliasesColor := lipgloss.Color("15")

	statusBarColor := lipgloss.Color("13")

	year := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Foreground(statusBarForeground).Background(yearColor).Bold(true).Render("2021")
	inReview := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Foreground(statusBarForeground).Background(inReviewColor).Bold(true).Render("In Review")
	commands := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Foreground(statusBarForeground).Background(commandsColor).Bold(true).Render(fmt.Sprintf("%v Commands", m.metrics.TotalCommands))
	atFig := lipgloss.NewStyle().PaddingLeft(1).PaddingRight(1).Foreground(statusBarForeground).Background(aliasesColor).Bold(true).Render("@fig")

	statusBarLeft := year + inReview
	statusBarRight := lipgloss.NewStyle().Background(statusBarColor).Width(physicalWidth - lipgloss.Width(statusBarLeft)).Align(lipgloss.Right).Render(commands + atFig)

	statusBar := statusBarLeft + statusBarRight

	if m.page == 0 {
		nextPage := lipgloss.NewStyle().
			MarginTop(2).
			Render(
				lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("[Press Enter to see your ") +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Bold(true).Render("#FigWrapped") +
					lipgloss.NewStyle().Foreground(lipgloss.Color("5")).Render("]"))

		fullPage = lipgloss.JoinVertical(lipgloss.Center, doc.String(), nextPage)
	} else {
		fullPage = doc.String()
	}

	if lipgloss.Width(fullPage) > physicalWidth || lipgloss.Height(fullPage) > physicalHeight {
		page := lipgloss.Place(
			physicalWidth,
			physicalHeight,
			lipgloss.Center,
			lipgloss.Center,
			"Expand your terminal to see your #FigWrapped!")

		return page
	}

	if true {
		page := lipgloss.Place(
			physicalWidth,
			physicalHeight,
			lipgloss.Center,
			lipgloss.Center,
			fullPage)

		return page
	} else {
		page := lipgloss.Place(
			physicalWidth,
			physicalHeight-1,
			lipgloss.Center,
			lipgloss.Center,
			fullPage)

		return lipgloss.JoinVertical(lipgloss.Center, page, statusBar)
	}

}

func NewCmdWrapped() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "wrapped",
		Short: "See your #FigWrapped",
		Long:  "What did you do in the terminal in 2021, find out with #FigWrapped!",
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
