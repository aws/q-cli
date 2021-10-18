#!/usr/bin/env bash
MAGENTA=$(tput setaf 5)
NORMAL=$(tput sgr0)

DISABLE_AUTOLAUNCH="$(fig settings app.disableAutolaunch)"
if [[ "${DISABLE_AUTOLAUNCH}" ==  "true" ]]; then
	: echo Autolaunch Disabled...
else 
	(fig launch > /dev/null &)
	echo "Launching ${MAGENTA}Fig${NORMAL}..."
	if [[ -z "${DISPLAYED_AUTOLAUNCH_SETTINGS_HINT}" ]]; then
	  echo "(To turn off autolaunch, run \`fig settings app.disableAutolaunch true\`)"
	  printf "\nDISPLAYED_AUTOLAUNCH_SETTINGS_HINT=1" >> ~/.fig/user/config
	fi
fi

