#!/usr/bin/env bash

# Read all the user defaults.
if [[ -s ~/.fig/user/config ]]; then
  source ~/.fig/user/config 
else
  exit
fi

if [[ "$FIG_LOGGED_IN" == '1' || -n $(defaults read com.mschrage.fig userEmail 2> /dev/null) ]]; then
  # If we are actually logged in, update accordingly and run onboarding campaign.
  sed -i '' "s/FIG_LOGGED_IN=.*/FIG_LOGGED_IN=1/g" ~/.fig/user/config 2> /dev/null
else
  exit
fi

if [[ "$FIG_ONBOARDING" == '0' ]] \
  && [[ "$TERM_PROGRAM" == "iTerm.app" || "$TERM_PROGRAM" == "Apple_Terminal" ]]; then
  fig app onboarding
  exit
fi

# User is logged in to Fig
MAGENTA=$(tput setaf 5)
NORMAL=$(tput sgr0)

FIG_IS_RUNNING="$(fig app running)"

# Ask for confirmation before updating
if [[ "${FIG_IS_RUNNING}" -eq 1 && ! -z "${NEW_VERSION_AVAILABLE}" ]]; then
    if [[ "$(fig settings app.disableAutoupdates)" ==  "true" ]]; then
        echo "A new version of ${MAGENTA}Fig${NORMAL} is available. (Autoupdates are disabled)"
    else 
        (fig update -y > /dev/null &)
        echo "Updating ${MAGENTA}Fig${NORMAL} to latest version..."
        (sleep 3 && fig app launch > /dev/null &)
        if [[ -z "${DISPLAYED_AUTOUPDATE_SETTINGS_HINT}" ]]; then
          echo "(To turn off automatic updates, run \`fig settings app.disableAutoupdates true\`)"
          printf "\nDISPLAYED_AUTOUPDATE_SETTINGS_HINT=1" >> ~/.fig/user/config
        fi
    fi
fi

if [[ -z "$APP_TERMINATED_BY_USER" && "${FIG_IS_RUNNING}" == '0' ]]; then
  if [[ "$(fig settings app.disableAutolaunch)" != "true" ]]; then
    (fig app launch > /dev/null &)
    echo "Launching ${MAGENTA}Fig${NORMAL}..."
    if [[ -z "${DISPLAYED_AUTOLAUNCH_SETTINGS_HINT}" ]]; then
      echo "(To turn off autolaunch, run \`fig settings app.disableAutolaunch true\`)"
      printf "\nDISPLAYED_AUTOLAUNCH_SETTINGS_HINT=1" >> ~/.fig/user/config
    fi
  fi
fi

# Show Fig tips
# Prevent termenv library from attempting to read color values and outputing random ANSI codes
# See https://github.com/muesli/termenv/blob/166cf3773788aab7e9bf5e34d8c0deb176b92bc8/termenv_unix.go#L172
# Disabled for now since it is VERRRYYYYY SLOW
# TERM=screen fig tips prompt 2>/dev/null

unset FIG_IS_RUNNING
