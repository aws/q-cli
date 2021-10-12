touch ~/.fig/ssh_hostnames
if ! cat ~/.fig/ssh_hostnames | grep -q $1; then

  BOLD=$(tput bold)
  UNDERLINE=$(tput smul)
  MAGENTA=$(tput setaf 5)
  NORMAL=$(tput sgr0)

  cat <<EOF
To install SSH support for ${MAGENTA}Fig${NORMAL}, run the following on your remote machine
  
  ${BOLD}${UNDERLINE}For bash/zsh:${NORMAL}
  source <(curl -Ls fig.io/ssh)

  ${BOLD}${UNDERLINE}For fish:${NORMAL}
  curl -Ls fig.io/ssh | source

EOF

  echo $1 >> ~/.fig/ssh_hostnames
fi
