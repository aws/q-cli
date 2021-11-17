package tips

import (
	"encoding/json"
	"fig-cli/logging"
	"fig-cli/settings"
	"fmt"
	"os"
	"os/user"
	"path/filepath"
	"regexp"
	"strconv"
	"time"

	"github.com/charmbracelet/lipgloss"
	"github.com/spf13/cobra"
)

type Tip struct {
	Id       string `json:"id"`
	Text     string `json:"text"`
	TipType  string `json:"type"`
	Priority int    `json:"priority"`
	Version  string `json:"version"`
	WaitTime int    `json:"waitTime"`
	Sent     bool   `json:"sent"`
}

type TipFile struct {
	TimeLastSent int   `json:"timeLastSent"`
	Queue        []Tip `json:"tipQueue"`
}

func (tipFile *TipFile) addTip(tip Tip) error {
	// Check if tip already exists
	for _, existingTip := range tipFile.Queue {
		if existingTip.Id == tip.Id {
			return fmt.Errorf("Tip already exists")
		}
	}

	// Add tip
	tipFile.Queue = append(tipFile.Queue, tip)
	return tipFile.saveTip()
}

func tipFilePath() (string, error) {
	user, err := user.Current()
	if err != nil {
		return "", err
	}

	tipFilePath := user.HomeDir + "/.fig/tips.json"
	return tipFilePath, nil
}

func (tip *TipFile) saveTip() error {
	tipFilePath, err := tipFilePath()
	if err != nil {
		return err
	}

	jsonString, err := json.MarshalIndent(tip, "", "  ")
	if err != nil {
		return err
	}

	jsonString = append(jsonString, []byte("\n")...)

	os.WriteFile(tipFilePath, jsonString, 0644)
	return nil
}

func loadTip() (*TipFile, error) {
	tipFilePath, err := tipFilePath()
	if err != nil {
		return nil, err
	}

	tipFile, err := os.Open(tipFilePath)
	if os.IsNotExist(err) {
		return &TipFile{
			TimeLastSent: int(time.Now().Unix()),
		}, nil
	}

	if err != nil {
		return nil, err
	}

	var tipFileData TipFile
	err = json.NewDecoder(tipFile).Decode(&tipFileData)
	if err != nil {
		return nil, err
	}

	return &tipFileData, nil
}

func NewCmdTips() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tips",
		Short: "Enable/Disable Fig tips",
	}

	cmd.AddCommand(NewCmdPrompt())
	cmd.AddCommand(NewCmdAddTip())
	cmd.AddCommand(NewCmdAddChangelog())
	cmd.AddCommand(NewCmdReset())
	cmd.AddCommand(NewCmdDisable())
	cmd.AddCommand(NewCmdEnable())

	return cmd
}

func NewCmdPrompt() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "prompt",
		Hidden: true,
		Run: func(cmd *cobra.Command, args []string) {
			settings, err := settings.Load()
			if err != nil {
				logging.Log("tips prompt", err.Error())
				return
			}

			if settings.Get("cli.tips.disabled") == true {
				return
			}

			tipFile, err := loadTip()
			if err != nil {
				logging.Log("tips prompt", err.Error())
				return
			}

			// Get unsent tips
			unsentTips := []Tip{}
			for _, tip := range tipFile.Queue {
				if !tip.Sent {
					unsentTips = append(unsentTips, tip)
				}
			}

			if len(unsentTips) == 0 {
				return
			}

			// Find max priority or type == "changelog"
			tipToSend := unsentTips[0]
			for _, tip := range unsentTips {
				if tip.Priority > tipToSend.Priority || tip.TipType == "changelog" {
					tipToSend = tip
				}
			}

			if tipToSend.Sent {
				return
			}

			// Check if to print tip
			if tipToSend.TipType == "changelog" {
				fmt.Println(tipToSend.Text)
			} else if tipToSend.TipType == "tip" {
				if int(time.Now().Unix())-tipFile.TimeLastSent > tipToSend.WaitTime {
					fmt.Println(tipToSend.Text)
				} else {
					return
				}
			} else {
				return
			}

			// Mark tip as sent
			for i, tip := range tipFile.Queue {
				if tip.Id == tipToSend.Id {
					tipFile.Queue[i].Sent = true
				}
			}

			tipFile.TimeLastSent = int(time.Now().Unix())

			err = tipFile.saveTip()
			if err != nil {
				logging.Log("tips prompt:", err.Error())
				return
			}
		},
	}

	return cmd
}

func NewCmdAddTip() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "add-tip [id] [priority] [wait-time] [text]",
		Hidden: true,
		Args:   cobra.ExactArgs(4),
		Run: func(cmd *cobra.Command, args []string) {
			tipFile, err := loadTip()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			priority, err := strconv.Atoi(args[1])
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			waitTime, err := strconv.Atoi(args[2])
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			tip := Tip{
				Id:       args[0],
				Text:     args[2],
				TipType:  "tip",
				Priority: priority,
				WaitTime: waitTime,
				Sent:     false,
			}

			err = tipFile.addTip(tip)
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		},
	}

	return cmd
}

func NewCmdAddChangelog() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "add-changelog [version] [text]",
		Hidden: true,
		Run: func(cmd *cobra.Command, args []string) {
			tipFile, err := loadTip()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			tip := Tip{
				Id:      fmt.Sprintf("changelog-%s", args[0]),
				Text:    args[1],
				TipType: "changelog",
				Version: args[0],
			}

			tipFile.Queue = append(tipFile.Queue, tip)
			err = tipFile.saveTip()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}
		},
	}

	return cmd
}

func NewCmdReset() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "reset",
		Hidden: true,
		Run: func(cmd *cobra.Command, args []string) {
			// Read ~/.fig/user/config
			user, err := user.Current()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			configPath := filepath.Join(user.HomeDir, ".fig", "user", "config")
			config, err := os.ReadFile(configPath)
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			tipsSent := true

			// Check for `FIG_ONBOARDING` in config
			onboardingRegex := regexp.MustCompile(`FIG_ONBOARDING\s*=\s*(0|1)`)
			if onboardingRegex.Match(config) {
				// Check value
				onboarding := onboardingRegex.FindStringSubmatch(string(config))[1]
				if onboarding == "0" {
					// Not onboarded
					tipsSent = false
				}
			}

			boldStyle := lipgloss.NewStyle().Bold(true)
			boldMagentaStyle := lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("5"))
			underlineStyle := lipgloss.NewStyle().Underline(true)

			tip1Text := "\n" + boldStyle.Render(`Fig Tips (1/5):`) +
				" 🚀 Customize keybindings\n\n" +
				"Fig lets you customize keybindings for:\n" +
				"  • inserting text (like tab/enter)\n" +
				"  • navigating (like " + boldStyle.Render("↑") + " & " + boldStyle.Render("↓") + " arrow keys)\n" +
				"  • toggling the description pop out (like ⌘+i)\n" +
				"  • and more\n\n" +
				"Just run " + boldMagentaStyle.Render("fig settings") + " and then select " + underlineStyle.Render("keybindings") + "\n\n" +
				underlineStyle.Render("Disable Fig Tips:") + " fig tips disable\n" +
				underlineStyle.Render("Report a bug:") + " fig issue\n"

			tip1 := Tip{
				Id:       "tip-1",
				Text:     tip1Text,
				TipType:  "tip",
				Priority: 10,
				// 10 minutes
				WaitTime: 60 * 10,
				Sent:     tipsSent,
			}

			tip2Text := "\n" + boldStyle.Render(`Fig Tips (2/5):`) +
				" ⚙️  Adjust settings\n\n" +
				"Customize autocomplete's look and feel for things like:\n" +
				"  • Width & height\n" +
				"  • Font family, font size, theme\n" +
				"  • Auto-execute functionality (e.g. allowing auto-execute after space)\n\n" +
				"Just run " + boldMagentaStyle.Render("fig settings") + "\n\n" +
				underlineStyle.Render("Disable Fig Tips:") + " fig tips disable\n" +
				underlineStyle.Render("Report a bug:") + " fig issue\n"

			tip2 := Tip{
				Id:       "tip-2",
				Text:     tip2Text,
				TipType:  "tip",
				Priority: 9,
				// 12 hours
				WaitTime: 60 * 60 * 12,
				Sent:     tipsSent,
			}

			tip3Text := "\n" + boldStyle.Render(`Fig Tips (3/5):`) +
				" 😎 Private autocomplete\n\n" +
				"Did you know Fig lets you private completions for your own personal shortcuts or even your team's internal CLI tool?\n\n" +
				"Build private completions in less than 2 minutes:\n" +
				"  1. " + boldStyle.Render("Personal:") + " " + underlineStyle.Render("fig.io/shortcuts") + "\n" +
				"  2. " + boldStyle.Render("Team:") + " " + underlineStyle.Render("fig.io/teams") + "\n\n" +
				underlineStyle.Render("Disable Fig Tips:") + " fig tips disable\n" +
				underlineStyle.Render("Report a bug:") + " fig issue\n"

			tip3 := Tip{
				Id:       "tip-3",
				Text:     tip3Text,
				TipType:  "tip",
				Priority: 8,
				// 12 hours
				WaitTime: 60 * 60 * 12,
				Sent:     tipsSent,
			}

			tip4Text := "\n" + boldStyle.Render(`Fig Tips (4/5):`) +
				" 🎉 Share Fig with friends\n\n" +
				"Enjoying Fig and think your friends & teammates would too?\n\n" +
				"Share Fig with friends!\n\n" +
				"Claim your custom invite link by running: " + boldMagentaStyle.Render("fig invite") + "\n\n" +
				underlineStyle.Render("Disable Fig Tips:") + " fig tips disable\n" +
				underlineStyle.Render("Report a bug:") + " fig issue\n"

			tip4 := Tip{
				Id:       "tip-4",
				Text:     tip4Text,
				TipType:  "tip",
				Priority: 7,
				// 12 hours
				WaitTime: 60 * 60 * 12,
				Sent:     tipsSent,
			}

			tip5Text := "\n" + boldStyle.Render(`Fig Tips (5/5):`) +
				" 🤗 Contribute to autocomplete for public CLIs\n\n" +
				"Missing completions for a CLI? Finding some errors in completions for an existing CLI?\n\n" +
				"All of Fig's completions for public CLI tools like cd, git, docker, kubectl are open source and community driven!\n\n" +
				"Start contributing at: " + underlineStyle.Render("github.com/withfig/autocomplete") + "\n\n" +
				underlineStyle.Render("Disable Fig Tips:") + " fig tips disable\n" +
				underlineStyle.Render("Report a bug:") + " fig issue\n"

			tip5 := Tip{
				Id:       "tip-5",
				Text:     tip5Text,
				TipType:  "tip",
				Priority: 6,
				// 12 hours
				WaitTime: 60 * 60 * 12,
				Sent:     tipsSent,
			}

			tipFile, err := loadTip()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			tips := []Tip{tip1, tip2, tip3, tip4, tip5}
			for _, tip := range tips {
				err := tipFile.addTip(tip)
				if err != nil {
					fmt.Printf("Error adding %s: %s\n", tip.Id, err)
				}
			}
		},
	}

	return cmd
}

func NewCmdDisable() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "disable",
		Short: "Disable Fig Tips",
		Run: func(cmd *cobra.Command, args []string) {
			settings, err := settings.Load()
			if err != nil {
				os.Exit(1)
			}

			settings.Set("cli.tips.disabled", true)
			err = settings.Save()
			if err != nil {
				os.Exit(1)
			}

			fmt.Printf("\n→ Fig Tips disabled...\n\n")
		},
	}

	return cmd
}

func NewCmdEnable() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "enable",
		Short: "Enable Fig Tips",
		Run: func(cmd *cobra.Command, args []string) {
			settings, err := settings.Load()
			if err != nil {
				os.Exit(1)
			}

			settings.Set("cli.tips.disabled", false)
			err = settings.Save()
			if err != nil {
				os.Exit(1)
			}

			fmt.Printf("\n→ Fig Tips enabled...\n\n")
		},
	}

	return cmd
}
