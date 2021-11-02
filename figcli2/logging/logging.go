package logging

import (
	"os"
	"os/user"

	"github.com/charmbracelet/lipgloss"
)

const (
	logFile = "/.fig/logs/cli.log"
)

type LoggingLevel int

const (
	LogLevelDebug LoggingLevel = iota
	LogLevelInfo
	LogLevelWarn
	LogLevelError
)

func Log(message ...string) error {
	user, err := user.Current()
	if err != nil {
		return err
	}

	f, err := os.OpenFile(user.HomeDir+logFile, os.O_APPEND|os.O_WRONLY|os.O_CREATE, 0644)
	if err != nil {
		return err
	}

	defer f.Close()

	_, err = f.WriteString(lipgloss.NewStyle().Foreground(lipgloss.Color("#0D98BA")).Render("CLI: "))
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
