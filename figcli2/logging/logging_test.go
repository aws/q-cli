package logging

import (
	"fmt"
	"os"
	"os/user"
	"strings"
	"testing"
)

func TestLog(t *testing.T) {
	Log("test from unit test")

	// Check log file has text in it
	u, _ := user.Current()
	file, err := os.ReadFile(u.HomeDir + logFile)
	if err != nil {
		t.Error("Failed to read log file")
	}

	fmt.Println(string(file))

	if !strings.Contains(string(file), "test from unit test") {
		t.Error("Log file does not contain expected text")
	}
}
