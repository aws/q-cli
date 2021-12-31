package logging

import (
	"os"
	"os/user"
	"path/filepath"
)

type LoggingLevel int

func GetLogFilepath() string {
	user, err := user.Current()
	if err != nil {
		return ""
	}

	return filepath.Join(user.HomeDir, ".fig", "logs", "cli.log")
}

const (
	LogLevelDebug LoggingLevel = iota
	LogLevelInfo
	LogLevelWarn
	LogLevelError
)

func Log(message ...string) error {
	logFilepath := GetLogFilepath()
	if logFilepath == "" {
		return nil
	}

	f, err := os.OpenFile(logFilepath, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0644)
	if err != nil {
		return err
	}

	defer f.Close()

	_, err = f.WriteString("CLI: ")
	if err != nil {
		return err
	}

	for i, m := range message {
		if i != 0 {
			_, err = f.WriteString(" ")
			if err != nil {
				return err
			}
		}

		_, err = f.WriteString(m)
		if err != nil {
			return err
		}
	}

	_, err = f.WriteString("\n")
	if err != nil {
		return err
	}

	return nil
}
