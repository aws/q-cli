package specs

import (
	"fmt"
	"os/user"
	"path/filepath"
	"strings"
)

func GetSpecsPaths() ([]string, error) {
	user, err := user.Current()
	if err != nil {
		return []string{}, err
	}

	// Get specs
	files, err := filepath.Glob(fmt.Sprintf("%s/.fig/autocomplete/*.js", user.HomeDir))
	if err != nil {
		return []string{}, err
	}

	return files, nil
}

func GetSpecsNames() ([]string, error) {
	specs, err := GetSpecsPaths()
	if err != nil {
		return []string{}, err
	}

	user, err := user.Current()
	if err != nil {
		return []string{}, err
	}

	// Trim the prefix and suffix
	for i, file := range specs {
		specs[i] = strings.TrimSuffix(strings.TrimPrefix(file, fmt.Sprintf("%s/.fig/autocomplete/", user.HomeDir)), ".js")
	}

	return specs, nil
}

func GetSpecsCount() (int, error) {
	specs, err := GetSpecsPaths()
	if err != nil {
		return 0, err
	}

	return len(specs), nil
}
