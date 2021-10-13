#!/usr/bin/env bash

GREEN='\033[0;32m'
RED='\033[0;31m'

echo -e "We will be sending the following files (if they exist) to hello@fig.io and will CC you:\n"
echo -e '  -' ~/.profile
echo -e '  -' ~/.zprofile
echo -e '  -' ~/.bash_profile
echo -e '  -' ~/.bashrc
echo -e '  -' ~/.zshrc
echo -e '  -' ~/.config/fish/config.fish
echo -e '\n'

function abort {
	echo -e "${RED}Aborting..."
	exit 1
}

read -rp "Continue? (Y/N): " confirm && [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]] || abort

echo -e '\n'

curl -X POST --data "$(defaults read com.mschrage.fig | grep userEmail)" \
	--data FILEBREAK --data-binary @"$HOME/.profile" \
	--data FILEBREAK --data-binary @"$HOME/.zprofile" \
	--data FILEBREAK --data-binary @"$HOME/.bash_profile" \
	--data FILEBREAK --data-binary @"$HOME/.bashrc" \
	--data FILEBREAK --data-binary @"$HOME/.zshrc" \
	--data FILEBREAK --data-binary @"$HOME/.config/fish/config.fish" \
	https://waitlist.withfig.com/dotfiles 2>/dev/null

echo -e "\n${GREEN}Dotfiles emailed!"
