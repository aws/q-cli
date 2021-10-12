#!/usr/bin/env bash

#  remote_cwd.sh
#  fig
#
#  Created by Matt Schrage on 1/7/21.
#  Copyright © 2021 Matt Schrage. All rights reserved.

# Do not delete the following comment:
# FIG_SETTINGS

# Heuristic for getting user's pid. Find all processes with a TTY and pick most recent.
pid=$(ps -e -o pid,tty,comm | grep -v "?" | grep 'zsh\|bash\|fish' | tail -n -1 | xargs | cut -d ' ' -f 1);

# https://stackoverflow.com/a/8597411/926887
if [ "$OSTYPE" == "linux-gnu"* ]; then
  readlink /proc/$pid/cwd
elif [ "$OSTYPE" == "darwin"* ]; then
  lsof -p $pid | grep cwd | awk '{print $9}'
elif [ "$OSTYPE" == "cygwin" ]; then
  pwd # not gonna handle this right now
elif [ "$OSTYPE" == "msys" ]; then
  pwd # not gonna handle this right now
elif [ "$OSTYPE" == "win32" ]; then
  pwd # not gonna handle this right now
elif [ "$OSTYPE" == "freebsd"* ]; then
  procstat -f $pid | grep cwd | awk '{print $10}'
else
  pwd # Unknown.
fi
