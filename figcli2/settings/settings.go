package settings

import (
	"encoding/json"
	"fmt"
	"os"
	"os/user"
)

type Settings map[string]interface{}

func (s Settings) Get(key string) interface{} {
	return s[key]
}

func (s Settings) Set(key string, value interface{}) {
	s[key] = value
}

func (s Settings) Delete(key string) {
	delete(s, key)
}

func New() Settings {
	return make(Settings)
}

// Load settings from a file
func Load() (Settings, error) {
	usr, err := user.Current()
	if err != nil {
		fmt.Println("Error: ", err)
		return nil, err
	}

	data, err := os.ReadFile(usr.HomeDir + "/.fig/settings.json")
	if err != nil {
		fmt.Println("Settings: settings file does not exist")
		return nil, err
	}

	var result map[string]interface{}
	err = json.Unmarshal(data, &result)

	if err != nil {
		fmt.Println("Settings: settings file is not valid JSON")
		return nil, err
	}

	return result, nil
}

func (s Settings) Save() error {
	usr, err := user.Current()
	if err != nil {
		fmt.Println("Error: ", err)
		return err
	}

	data, err := json.MarshalIndent(s, "", "  ")
	if err != nil {
		fmt.Println("Settings: error marshalling settings")
		return err
	}

	err = os.WriteFile(usr.HomeDir+"/.fig/settings.json", data, 0644)
	if err != nil {
		fmt.Println("Settings: error writing settings file")
		return err
	}

	return nil
}

func GetFilepath() (string, error) {
	usr, err := user.Current()
	if err != nil {
		return "", err
	}

	return usr.HomeDir + "/.fig/settings.json", nil
}
