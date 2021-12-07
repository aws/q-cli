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

	"github.com/spf13/cobra"
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

type HistoryMetrics struct {
	TopCommandsUsage       []CommandUsage
	TopWorkingDirs         []WorkingDirUsage
	TopShells              []ShellUsage
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
		if command != "" {
			commandsUsageMap[command]++
		}

		if shellAliases[h.Shell] != nil {
			if shellAliases[h.Shell][command] != "" {
				charsSavedByAlias += len(shellAliases[h.Shell][command]) - len(command)
			}
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

	return metrics
}

func NewCmdWrapped() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "wrapped",
		Short:  "How did you use the shell in 2021",
		Long:   "How did you use the shell in 2021",
		Hidden: true,
		Run: func(cmd *cobra.Command, arg []string) {
			history, _ := loadHistory()
			metrics := Metrics(history)

			fmt.Printf("\n\nHere is your 2021 in the shell wrapped up:\n\n")

			// Command usage
			maxCommands := 10
			if len(metrics.TopCommandsUsage) < maxCommands {
				maxCommands = len(metrics.TopCommandsUsage)
			}

			fmt.Printf("\nTop %v commands:\n\n", maxCommands)
			for _, command := range metrics.TopCommandsUsage[0:maxCommands] {
				fmt.Printf("%5v: %v\n", command.Count, command.Command)
			}

			// Working dirs
			maxWorkingDirs := 5
			if len(metrics.TopWorkingDirs) < maxWorkingDirs {
				maxWorkingDirs = len(metrics.TopWorkingDirs)
			}

			fmt.Printf("\nTop %v working dirs:\n\n", maxWorkingDirs)
			for _, workingDir := range metrics.TopWorkingDirs[0:maxWorkingDirs] {
				// Pretty print working dir
				user, _ := user.Current()

				workingDirPretty := strings.Replace(workingDir.WorkingDir, user.HomeDir, "~", 1)
				if workingDirPretty[len(workingDirPretty)-1] != '/' {
					workingDirPretty += "/"
				}

				fmt.Printf("%5v: %v\n", workingDir.Count, workingDirPretty)
			}

			fmt.Printf("\nLongest Piped Sequence:\n")
			fmt.Printf(" %v\n", metrics.LongestPipedSequence)

			// Time of Day Histogram
			fmt.Printf("\nTime of Day Histogram:\n\n")

			maxCount := 0
			for _, count := range metrics.MostCommonTimeOfDay {
				if count > maxCount {
					maxCount = count
				}
			}

			for i := 0; i < 24; i++ {
				time := ""
				if i == 0 {
					time = fmt.Sprintf("%3d AM", 12)
				} else if i <= 12 {
					time = fmt.Sprintf("%3d AM", i)
				} else if i == 12 {
					time = fmt.Sprintf("%3d PM", i)
				} else {
					time = fmt.Sprintf("%3d PM", i-12)
				}

				fmt.Printf("%v %v\n", time, strings.Repeat("█", int(float64(metrics.MostCommonTimeOfDay[i])/float64(maxCount)*70)))
			}

			// Day of Week Histogram
			fmt.Printf("\nDay of Week Histogram:\n\n")

			maxCount = 0
			for _, count := range metrics.MostCommonDayOfWeek {
				if count > maxCount {
					maxCount = count
				}
			}

			for i := 0; i < 7; i++ {
				fmt.Printf("%10v %v\n",
					time.Weekday(i),
					strings.Repeat("█", int(float64(metrics.MostCommonDayOfWeek[time.Weekday(i)])/float64(maxCount)*66)))
			}

			maxShells := 3
			if len(metrics.TopShells) < maxShells {
				maxShells = len(metrics.TopShells)
			}

			// Top Shells
			fmt.Printf("\nTop Shells:\n\n")
			for _, shell := range metrics.TopShells[0:maxShells] {
				if shell.Count > 0 {
					fmt.Printf("%5v: %v\n", shell.Count, shell.Shell)
				}
			}

			// Shell Aliases
			fmt.Printf("\nChars saved by alias: %v\n\n", metrics.CharactersSavedByAlias)
		},
	}

	return cmd
}
