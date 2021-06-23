#!/bin/sh
# Adapted from https://www.thegeekstuff.com/2009/09/multitail-to-view-tail-f-output-of-multiple-log-files-in-one-terminal/
# When this exits, exit all back ground process also.
#trap 'kill $(jobs -p)' EXIT
#
#echo "$@"
## iterate through the each given file names,
#for file in "$@"
#do
#  # show tails of each in background.
#  tail -f "$HOME/.fig/logs/$file.log" &
#done
#
## wait .. until CTRL+C
#wait

 exec tail -qf ~/.fig/logs/*
