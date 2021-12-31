package diagnostics

import "testing"

func TestInstalledViaBrew(t *testing.T) {
	viaBrew, err := InstalledViaBrew()
	if err != nil {
		t.Errorf("Error getting mac os version: %s", err.Error())
	}

	t.Logf("Mac os version: %t", viaBrew)
}
func TestGetMacOsVersion(t *testing.T) {
	macosVersion, err := GetMacOsVersion()
	if err != nil {
		t.Errorf("Error getting mac os version: %s", err.Error())
	}

	if macosVersion == "" {
		t.Errorf("Mac os version is empty")
	}

	t.Logf("Mac os version: %s", macosVersion)
}

func TestReadPlist(t *testing.T) {
	getVersion, err := GetFigVersion()
	if err != nil {
		t.Errorf("Error getting fig version: %s", err.Error())
	}

	if getVersion == "" {
		t.Errorf("Fig version is empty")
	}

	t.Logf("Fig version: %s", getVersion)

	getBuild, err := GetFigBuild()
	if err != nil {
		t.Errorf("Error getting fig build: %s", err.Error())
	}

	if getBuild == "" {
		t.Errorf("Fig build is empty")
	}

	t.Logf("Fig build: %s", getBuild)
}

func TestAppInfo(t *testing.T) {
	appInfo, err := GetAppInfo()
	if err != nil {
		t.Errorf("Error getting app info: %s", err.Error())
	}

	path, err := appInfo.BundlePath()
	if err != nil {
		t.Errorf("Error getting bundle path: %s", err.Error())
	}

	if path == "" {
		t.Errorf("Bundle path is empty")
	}

	t.Logf("Bundle path: %s", path)

	version, err := appInfo.BuildVersion()
	if err != nil {
		t.Errorf("Error getting build version: %s", err.Error())
	}

	if version == "" {
		t.Errorf("Build version is empty")
	}

	t.Logf("Build version: %s", version)

	pid, err := appInfo.Pid()
	if err != nil {
		t.Errorf("Error getting pid: %s", err.Error())
	}

	if pid == 0 {
		t.Errorf("Pid is 0")
	}

	t.Logf("Pid: %d", pid)
}
