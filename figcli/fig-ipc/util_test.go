package fig_ipc

import "testing"

func TestGetShell(t *testing.T) {
	shell, err := GetShell()
	if err != nil {
		t.Error(err)
	}

	if shell == "" {
		t.Error("shell is empty")
	}

	t.Log(shell)
}

func TestGetCurrentTerminal(t *testing.T) {
	term, err := GetCurrentTerminal()
	if err != nil {
		t.Error(err)
	}

	if term == "" {
		t.Error("terminal is empty")
	}

	t.Log(term)

	potentialBundle, err := term.PotentialBundleId()
	if err != nil {
		t.Error(err)
	}

	if potentialBundle == "" {
		t.Error("potential bundle is empty")
	}

	t.Log(potentialBundle)
}
