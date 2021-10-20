package main

import (
	"fig-cli/cmd"
	fig_ipc "fig-cli/fig-ipc"
	"fig-cli/proto"
)

func main() {
	cmd.Execute()
	// ipc, err := fig_ipc.Connect()
	// if err != nil {
	// 	panic(err)
	// }

	// buffer := &fig_proto.EditBuffer{
	// 	Text:      "Hello World",
	// 	Cursor:    1,
	// 	Shell:     "bash",
	// 	SessionId: "123",
	// }

	// ipc.SendFigProto(buffer)
	// ipc.SendFigJson(`{"Hello": "test"}`)

	// res, _ := ipc.Recv()

	// fmt.Println(res)

	// auth, _ := fig_teams.GetAuthToken()
	// teams, _ := fig_teams.FetchTeams(auth)
	// for _, team := range teams {
	// 	fmt.Println(team.(map[string]interface{})["name"])
	// }
	// who, _ := fig_teams.FetchWhoAmI(auth)
	// fmt.Println(who)
	// teams.PostTeam(auth)

	conn, _ := fig_ipc.Connect()
	b, t, a := conn.RecvMessage()
}
