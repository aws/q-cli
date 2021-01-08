#!/usr/bin/env bash

#  help.sh
#  fig
#
#  Created by Matt Schrage on 1/6/21.
#  Copyright © 2021 Matt Schrage. All rights reserved.
BOLD=$(tput bold)
NORMAL=$(tput sgr0)

cat <<EOF
CLI to interact with Fig

${BOLD}USAGE${NORMAL}
  $ fig [COMMAND]

${BOLD}COMMANDS${NORMAL}
  invite        invite up to 5 friends & teammates to Fig
  update        update repo of completion scripts
  docs          documentation for building completion specs
  source        (re)connect fig to the current shell session
  list          print all commands with completion specs
  uninstall     remove a completion spec
  --help        a summary of the Fig CLI commands
  
EOF
