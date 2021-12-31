package teams

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os/exec"
	"strings"
)

const baseURL = "https://node-backend-mkoj.onrender.com"

type AuthToken string

func GetAuthToken() (AuthToken, error) {
	authExec, err := exec.Command("defaults", "read", "com.mschrage.fig", "access_token").Output()
	if err != nil {
		return AuthToken(""), err
	}
	authToken := strings.TrimSpace(string(authExec))

	return AuthToken(authToken), nil
}

func Fetch(authToken AuthToken, endpoint string) ([]byte, error) {
	req, _ := http.NewRequest("GET", baseURL+endpoint, nil)
	req.Header.Set("Authorization", "Bearer "+string(authToken))
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{}

	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)

	if err != nil {
		return nil, err
	}

	if resp.StatusCode > 299 {
		return nil, fmt.Errorf("%s", body)
	}

	return body, nil
}

func Post(authToken AuthToken, endpoint string, body []byte) ([]byte, error) {
	req, _ := http.NewRequest("POST", baseURL+endpoint, bytes.NewReader(body))
	req.Header.Set("Authorization", "Bearer "+string(authToken))
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{}

	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	defer resp.Body.Close()

	body, err = io.ReadAll(resp.Body)

	if err != nil {
		return nil, err
	}

	if resp.StatusCode > 299 {
		return body, fmt.Errorf("%s", string(body))
	}

	return body, nil
}

func Delete(authToken AuthToken, endpoint string, body []byte) ([]byte, error) {
	req, _ := http.NewRequest("DELETE", baseURL+endpoint, bytes.NewReader(body))
	req.Header.Set("Authorization", "Bearer "+string(authToken))
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{}

	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}

	defer resp.Body.Close()

	body, err = io.ReadAll(resp.Body)

	if err != nil {
		return nil, err
	}

	if resp.StatusCode > 299 {
		return body, fmt.Errorf("%s", string(body))
	}

	return body, nil
}

func FetchTeams(authToken AuthToken) ([]interface{}, error) {
	teamsRes, err := Fetch(authToken, "/teams")
	if err != nil {
		return nil, err
	}

	var teams []interface{}
	json.Unmarshal(teamsRes, &teams)

	return teams, nil
}

func PostTeam(authToken AuthToken, name string) (string, error) {
	escapedName := strings.Replace(name, "\"", "\\\"", -1)

	res, err := Post(authToken, "/teams", []byte(fmt.Sprintf(`{"name":"%s"}`, escapedName)))
	if err != nil {
		return "", err
	}

	var teamJson map[string]interface{}
	json.Unmarshal(res, &teamJson)

	if teamJson["name"] == nil {
		return "", fmt.Errorf("no team name in response")
	}

	return teamJson["name"].(string), nil
}

func FetchWhoAmI(authToken AuthToken) (string, error) {
	whoami, err := Fetch(authToken, "/teams/me")
	if err != nil {
		return "", err
	}

	whoamiStr := string(whoami)
	return whoamiStr, nil
}

func FetchTeamMembers(authToken AuthToken, teamName string) ([]interface{}, error) {
	membersRes, err := Fetch(authToken, fmt.Sprintf("/teams/%s/users", teamName))
	if err != nil {
		return nil, err
	}

	var members []interface{}
	json.Unmarshal(membersRes, &members)

	return members, nil
}

func PostTeamMember(authToken AuthToken, teamName string, email string) error {
	escapedEmail := strings.Replace(email, "\"", "\\\"", -1)

	_, err := Post(authToken, fmt.Sprintf("/teams/%s/users", teamName), []byte(fmt.Sprintf(`{"emailToAdd":"%s"}`, escapedEmail)))
	if err != nil {
		return err
	}

	return nil
}

func DeleteTeamMember(authToken AuthToken, teamName string, email string) error {
	escapedEmail := strings.Replace(email, "\"", "\\\"", -1)

	res, err := Delete(authToken, fmt.Sprintf("/teams/%s/users", teamName), []byte(fmt.Sprintf(`{"emailToRemove":"%s"}`, escapedEmail)))
	if err != nil {
		return err
	}

	fmt.Println(err)
	fmt.Println(string(res))
	return nil
}


func PatchTeamMemberRole(authToken AuthToken, teamName string, email string, role string) error {
	escapedEmail := strings.Replace(email, "\"", "\\\"", -1)
	escapedRole := strings.Replace(role, "\"", "\\\"", -1)

	_, err := Post(authToken, fmt.Sprintf("/teams/%s/users", teamName), []byte(fmt.Sprintf(`{"emailToAdd":"%s", "role":"%s"}`, escapedEmail, escapedRole)))
	if err != nil {
		return err
	}

	return nil
}