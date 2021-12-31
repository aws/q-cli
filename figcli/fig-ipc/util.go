package fig_ipc

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
)

// NOTE: this does not work when using `go run`, you must use `go build`
// since it is getting the parent ppid
func GetShell() (string, error) {
	parentId := os.Getppid()
	execPs, err := exec.Command("ps", "-p", fmt.Sprintf("%d", parentId), "-o", "comm=").Output()
	if err != nil {
		return "unknown", err
	}

	return strings.TrimSpace(string(execPs)), nil
}

type Terminal string

func GetCurrentTerminal() (Terminal, error) {
	if os.Getenv("KITTY_WINDOW_ID") != "" {
		return Terminal("kitty"), nil
	}

	if os.Getenv("ALACRITTY_LOG") != "" {
		return Terminal("alacritty"), nil
	}

	if strings.Contains(os.Getenv("TERM_PROGRAM_VERSION"), "insider") {
		return Terminal("vscode-insiders"), nil
	}

	term := os.Getenv("TERM_PROGRAM")
	if term == "" {
		return Terminal("unknown"), fmt.Errorf("could not determine terminal")
	}

	return Terminal(term), nil
}

func (t Terminal) PotentialBundleId() (string, error) {
	switch t {
	case Terminal("vscode-insiders"):
		return "com.microsoft.VSCodeInsiders", nil
	case Terminal("vscode"):
		return "com.microsoft.VSCode", nil
	case Terminal("Apple_Terminal"):
		return "com.apple.Terminal", nil
	case Terminal("Hyper"):
		return "co.zeit.hyper", nil
	case Terminal("iTerm.app"):
		return "com.googlecode.iterm2", nil
	}

	termBundle := os.Getenv("TERM_BUNDLE_IDENTIFIER")

	if termBundle == "" {
		return "unknown", fmt.Errorf("could not determine terminal bundle")
	}

	return termBundle, nil
}
