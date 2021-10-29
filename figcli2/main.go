package main

import (
	"fig-cli/cmd"
	"fig-cli/logging"
	"os"
	"strings"
)

func main() {
	_ = logging.Log("Executing:", strings.Join(os.Args, " "))
	cmd.Execute()
}
