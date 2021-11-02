package main

import (
	"fig-cli/cmd/root"
	"fig-cli/logging"
	"os"
	"strings"
)

func main() {
	_ = logging.Log("Executing:", strings.Join(os.Args, " "))
	root.Execute()
}
