package cmd

import (
	"fig-cli/teams"
	"fmt"

	"github.com/spf13/cobra"
)

func init() {
	teamsCmd.AddCommand(teamsCreateCmd)
	teamsCmd.AddCommand(teamsLsCmd)
	teamsCmd.AddCommand(teamsMembersCmd)

	teamsMembersCmd.AddCommand(teamsMemberAddCmd)
	teamsMembersCmd.AddCommand(teamsMemberRemoveCmd)
	teamsMembersCmd.AddCommand(teamsMemberRoleCmd)

	rootCmd.AddCommand(teamsCmd)
}

var teamsCmd = &cobra.Command{
	Use:   "teams",
	Short: "Collaborate with your team using Fig",
	Long:  "Collaborate with your team using Fig",
	Annotations: map[string]string{
		"figcli.command.categories": "Common",
	},
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("teams called")
	},
}

var teamsCreateCmd = &cobra.Command{
	Use:   "create",
	Short: "Create a new team",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		auth, err := teams.GetAuthToken()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		teamName, err := teams.PostTeam(auth, args[0])
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		fmt.Println("Team created: ", teamName)
	},
}

var teamsLsCmd = &cobra.Command{
	Use:   "ls",
	Short: "List all teams for the current user",
	Run: func(cmd *cobra.Command, args []string) {
		auth, err := teams.GetAuthToken()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		teams, err := teams.FetchTeams(auth)
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		for _, team := range teams {
			fmt.Println(team.(map[string]interface{})["name"])
		}
	},
}

var teamsMembersCmd = &cobra.Command{
	Use:   "members [team]",
	Short: "Manage the members of the team",
	Long:  "List the members of the team and their roles.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		auth, err := teams.GetAuthToken()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		members, err := teams.FetchTeamMembers(auth, args[0])
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		fmt.Println(len(members), "members")

		for _, member := range members {
			fmt.Printf("%s - %s\n", member.(map[string]interface{})["email"], member.(map[string]interface{})["role"])
		}
	},
}

var teamsMemberAddCmd = &cobra.Command{
	Use:   "add [team] [email]",
	Short: "Add a member to a team",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		auth, err := teams.GetAuthToken()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		err = teams.PostTeamMember(auth, args[0], args[1])
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		fmt.Println("Member added")
	},
}

var teamsMemberRemoveCmd = &cobra.Command{
	Use:   "remove [team] [email]",
	Short: "Remove a member from a team",
	Args:  cobra.ExactArgs(2),
	Run: func(cmd *cobra.Command, args []string) {
		auth, err := teams.GetAuthToken()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		err = teams.DeleteTeamMember(auth, args[0], args[1])
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		fmt.Println("Member removed")
	},
}

var teamsMemberRoleCmd = &cobra.Command{
	Use:   "role [team] [email] [role]",
	Short: "Set a member's role in a team",
	Args:  cobra.ExactArgs(3),
	Run: func(cmd *cobra.Command, args []string) {
		auth, err := teams.GetAuthToken()
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		err = teams.PatchTeamMemberRole(auth, args[0], args[1], args[2])
		if err != nil {
			fmt.Println("Error:", err)
			return
		}

		fmt.Println("Member role set")
	},
}
