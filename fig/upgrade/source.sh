MAGENTA=$(tput setaf 5)
BOLD=$(tput bold)
NORMAL=$(tput sgr0)

echo "${MAGENTA}${BOLD}fig${NORMAL} is now connected to this terminal session. ($(tty))"
fig bg:init $SHELLPID $(tty)
#fig bg:cd
