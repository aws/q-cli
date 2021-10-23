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
