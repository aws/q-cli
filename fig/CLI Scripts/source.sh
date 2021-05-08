MAGENTA=$(tput setaf 5)
BOLD=$(tput bold)
NORMAL=$(tput sgr0)

fig settings:init > /dev/null
printf "\n${MAGENTA}${BOLD}fig${NORMAL} is now connected to this terminal session. ($(tty))\n\n"
fig bg:init $SHELLPID $(tty)
