package tweet

import (
	"fmt"
	"os/exec"

	"github.com/spf13/cobra"
)

func NewCmdTweet() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tweet",
		Short: "Tweet about Fig",
		Long:  `Tweet about Fig`,
		Annotations: map[string]string{
			"figcli.command.categories": "Common",
		},
		Run: func(cmd *cobra.Command, arg []string) {
			fmt.Printf("\n→ Opening Twitter...\n\n")
			err := exec.Command("open", "https://twitter.com/intent/tweet?text=I%27ve%20added%20autocomplete%20to%20my%20terminal%20using%20@fig!%0a%0a%F0%9F%9B%A0%F0%9F%86%95%F0%9F%91%89%EF%B8%8F&url=https://fig.io").Run()
			if err != nil {
				fmt.Println(err)
			}
		},
	}

	return cmd
}
