package teams

import "testing"

func TestGetAuthToken(t *testing.T) {
	authToken, err := GetAuthToken()
	if err != nil {
		t.Errorf("Error getting auth token: %s", err.Error())
	}

	if authToken == "" {
		t.Error("Auth token is empty")
	}
}
