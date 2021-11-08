package drip

import (
	"encoding/json"
	"fmt"
	"os"
	"os/user"
	"strconv"
	"time"

	"github.com/spf13/cobra"
)

type Drip struct {
	Text     string `json:"text"`
	DripType string `json:"type"`
	Priority int    `json:"priority"`
	Version  string `json:"version"`
	WaitTime int    `json:"waitTime"`
}

type DripFile struct {
	TimeLastSent int    `json:"timeLastSent"`
	Queue        []Drip `json:"dripQueue"`
}

func dripFilePath() (string, error) {
	user, err := user.Current()
	if err != nil {
		return "", err
	}

	dripFilePath := user.HomeDir + "/.fig/drip.json"
	return dripFilePath, nil
}

func (drip *DripFile) saveDrip() error {
	dripFilePath, err := dripFilePath()
	if err != nil {
		return err
	}

	jsonString, err := json.MarshalIndent(drip, "", "  ")
	if err != nil {
		return err
	}

	jsonString = append(jsonString, []byte("\n")...)

	os.WriteFile(dripFilePath, jsonString, 0644)
	return nil
}

func loadDrip() (*DripFile, error) {
	dripFilePath, err := dripFilePath()
	if err != nil {
		return nil, err
	}

	dripFile, err := os.Open(dripFilePath)
	if os.IsNotExist(err) {
		return &DripFile{}, nil
	}

	if err != nil {
		return nil, err
	}

	var dripFileData DripFile
	err = json.NewDecoder(dripFile).Decode(&dripFileData)
	if err != nil {
		return nil, err
	}

	return &dripFileData, nil
}

func NewCmdDrip() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "drip",
		Short:  "",
		Hidden: true,
		Run: func(cmd *cobra.Command, args []string) {
			dripFile, err := loadDrip()
			if err != nil {
				fmt.Println(err)
				return
			}

			if len(dripFile.Queue) == 0 {
				return
			}

			// Find max priority or type == "changelog"
			dripToSend := dripFile.Queue[0]
			dripIndex := 0
			for i, drip := range dripFile.Queue {
				if drip.Priority > dripToSend.Priority || drip.DripType == "changelog" {
					dripToSend = drip
					dripIndex = i
				}
			}

			// Check if to print drip
			if dripToSend.DripType == "changelog" {
				fmt.Println(dripToSend.Text)
			} else if dripToSend.DripType == "drip" {
				if int(time.Now().Unix())-dripFile.TimeLastSent > dripToSend.WaitTime {
					fmt.Println(dripToSend.Text)
				} else {
					return
				}
			} else {
				return
			}

			// Remove drip from queue
			dripFile.Queue = append(dripFile.Queue[:dripIndex], dripFile.Queue[dripIndex+1:]...)
			dripFile.TimeLastSent = int(time.Now().Unix())
			err = dripFile.saveDrip()
			if err != nil {
				fmt.Println(err)
				return
			}
		},
	}

	cmd.AddCommand(NewCmdAddDrip())

	return cmd
}

func NewCmdAddDrip() *cobra.Command {
	cmd := &cobra.Command{
		Use:    "add-drip [priority] [wait-time] [text]",
		Hidden: true,
		Args:   cobra.ExactArgs(3),
		Run: func(cmd *cobra.Command, args []string) {
			dripFile, err := loadDrip()
			if err != nil {
				fmt.Println(err)
				return
			}

			priority, err := strconv.Atoi(args[0])
			if err != nil {
				fmt.Println(err)
				return
			}

			waitTime, err := strconv.Atoi(args[1])
			if err != nil {
				fmt.Println(err)
				return
			}

			drip := Drip{
				Text:     args[2],
				DripType: "drip",
				Priority: priority,
				WaitTime: waitTime,
			}

			dripFile.Queue = append(dripFile.Queue, drip)
			err = dripFile.saveDrip()
			if err != nil {
				fmt.Println(err)
				return
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
			dripFile, err := loadDrip()
			if err != nil {
				fmt.Println(err)
				return
			}

			drip := Drip{
				Text:     args[1],
				DripType: "changelog",
				Version:  args[0],
			}

			dripFile.Queue = append(dripFile.Queue, drip)
			err = dripFile.saveDrip()
			if err != nil {
				fmt.Println(err)
				return
			}
		},
	}

	return cmd
}
