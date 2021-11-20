package fig_ipc

import "testing"

func TestGetIntegrations(t *testing.T) {
	res, err := GetIntegrations()
	if err != nil {
		t.Error(err)
	}

	if len(res) == 0 {
		t.Error("No integrations found")
	}

	for _, i := range res {
		if i.Name == "" {
			t.Error("Integration name is empty")
		}

		if i.BundleIdentifier == "" {
			t.Error("Integration bundle identifier is empty")
		}

		if *i.Status == "" {
			t.Error("Integration status is empty")
		}
	}
}
