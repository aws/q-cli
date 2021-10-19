package main

import (
	"fig-cli/cmd"
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
}
