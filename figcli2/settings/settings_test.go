package settings

import (
	"os"
	"testing"
)

func TestSettings(t *testing.T) {
	settingsFilepath, err := GetFilepath()
	if err != nil {
		t.Errorf("Error getting settings filepath: %s", err.Error())
	}

	if _, err := os.Stat(settingsFilepath); err != nil {
		t.Logf("Settings file does not exist: %s", settingsFilepath)
		os.WriteFile(settingsFilepath, []byte("{}"), 0644)
	}

	// Read settings file and save to var
	settingRaw, err := os.ReadFile(settingsFilepath)
	if err != nil {
		t.Errorf("Error reading settings file: %s", err.Error())
	}

	restoreSettings := func() {
		err := os.WriteFile(settingsFilepath, settingRaw, 0644)
		if err != nil {
			t.Errorf("Error restoring settings file: %s", err.Error())
		}
	}

	defer restoreSettings()

	settings, err := Load()
	if err != nil {
		t.Errorf("Error loading settings: %s", err.Error())
	}

	// Test setting
	settings.Set("test", "test")
	if settings.Get("test") != "test" {
		t.Errorf("Error setting settings")
	}

	// Test getting
	if settings.Get("test") != "test" {
		t.Errorf("Error getting settings")
	}

	// Test getting non-existent
	if settings.Get("test2") != nil {
		t.Errorf("Error getting non-existent settings")
	}
}
