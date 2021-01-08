#!/usr/bin/env bash

#  uninstall_spec.sh
#  fig
#
#  Created by Matt Schrage on 1/6/21.
#  Copyright © 2021 Matt Schrage. All rights reserved.
BOLD=$(tput bold)
NORMAL=$(tput sgr0)

SPEC=$1

if [[ $# -ne 1 ]]; then

cat <<EOF
  ${BOLD}Usage${NORMAL}
  fig uninstall <completion>

EOF
exit


fi

rm -rf ~/.fig/autocomplete/$SPEC.js

cat <<EOF
  
  ${BOLD}Uninstalled completions for ${SPEC}${NORMAL}!
  
  If something is broken or can be improved:

› Email hello@withfig.com or create an issue at github.com/withfig/autocomplete

› If you want to delete multiple specs at a time, open ~/.fig/autocomplete/
  
EOF
