package theme

import (
	"strings"
	"testing"
)

func test_theme(t *testing.T) {
	theme, err := getTheme()
	if err != nil {
		t.Error(err)
	}

	setResult, err := setTheme(theme)
	if err != nil {
		t.Error(err)
	}

	if strings.Contains(setResult, "Switching to theme") == false {
		t.Error("Failed to set theme")
	}
}
