package logging

import (
	"fmt"
	"os"
	"strings"
	"testing"
)

func TestLog(t *testing.T) {
	Log("test from unit test")

	// Check log file has text in it
	file, err := os.ReadFile(GetLogFilepath())
	if err != nil {
		t.Error("Failed to read log file")
	}

	fmt.Println(string(file))

	if !strings.Contains(string(file), "test from unit test") {
		t.Error("Log file does not contain expected text")
	}
}
