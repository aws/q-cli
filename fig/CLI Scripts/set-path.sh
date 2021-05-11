#!/bin/sh

#  set-path.sh
#  fig
#
#  Created by Matt Schrage on 5/7/21.
#  Copyright © 2021 Matt Schrage. All rights reserved.
printf '\n  Setting $PATH variable in Fig pseudo-terminal...\n\n'

# Update settings.json
fig settings pty.path "$PATH"

# Trigger update of ENV in PTY
fig bg:init $SHELLPID $(tty)
