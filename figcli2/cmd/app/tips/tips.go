package tips

import (
	"encoding/json"
	"fig-cli/settings"
	"fmt"
	"os"
	"os/user"
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

	tipFilePath := user.HomeDir + "/.fig/tip.json"
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
		return &TipFile{}, nil
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

func NewCmdTip() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "tip",
		Short:  "",
		Hidden: true,
		Run: func(cmd *cobra.Command, args []string) {
			settings, err := settings.Load()
			if err != nil {
				return
			}

			if settings.Get("cli.tip.disabled") == true {
				return
			}

			tipFile, err := loadTip()
			if err != nil {
				fmt.Println(err)
				return
			}

			if len(tipFile.Queue) == 0 {
				return
			}

			// Find max priority or type == "changelog"
			tipToSend := tipFile.Queue[0]
			tipIndex := 0
			for i, tip := range tipFile.Queue {
				if tip.Priority > tipToSend.Priority || tip.TipType == "changelog" {
					tipToSend = tip
					tipIndex = i
				}
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
			tipFile.Queue[tipIndex].Sent = true
			tipFile.TimeLastSent = int(time.Now().Unix())
			err = tipFile.saveTip()
			if err != nil {
				fmt.Println(err)
				return
			}
		},
	}

	cmd.AddCommand(NewCmdAddTip())
	cmd.AddCommand(NewCmdAddChangelog())
	cmd.AddCommand(NewCmdReset())

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
			boldStyle := lipgloss.NewStyle().Bold(true)

			tip1Text := boldStyle.Render(`Fig Tips (1/5):`) + ` Selecting Files / Folders

When selecting a file or folder:
  * Type $ ` + boldStyle.Render("~/") + ` to start autocompleting from the home directory 
  * Type $ ` + boldStyle.Render("../") + ` to start autocompleting from the directory above

` + boldStyle.Render("To disable Fig Tips: ") + "fig tips disable"

			tip1 := Tip{
				Id:       "tip-1",
				Text:     tip1Text,
				TipType:  "tip",
				Priority: 10,
				WaitTime: 43200,
			}

			tip2Text := boldStyle.Render(`Fig Tips (2/5):`) + ` Selecting Files / Folders

When selecting a file or folder:
  * Type $ ` + boldStyle.Render("~/") + ` to start autocompleting from the home directory 
  * Type $ ` + boldStyle.Render("../") + ` to start autocompleting from the directory above

` + boldStyle.Render("To disable Fig Tips: ") + "fig tips disable"

			tip2 := Tip{
				Id:       "tip-2",
				Text:     tip2Text,
				TipType:  "tip",
				Priority: 9,
				WaitTime: 43200,
			}

			tipFile, err := loadTip()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			err = tipFile.addTip(tip1)
			if err != nil {
				fmt.Println("Error adding tip-1:", err)
			}

			err = tipFile.addTip(tip2)
			if err != nil {
				fmt.Println("Error adding tip-2:", err)
			}
		},
	}

	return cmd
}
