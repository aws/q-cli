#!/usr/bin/env bash

# invariants:
# 1. Fig.app is installed
# 2. fig CLI tool is in $PATH
# 3. prompts.sh will only be called once per terminal session

if [[ "$FIG_CHECKED_PROMPTS" == "1" ]]; then
  exit
fi

# Read all the user defaults.
if [[ -s ~/.fig/user/config ]]; then
  source ~/.fig/user/config 
else
  exit
fi

# Set the tips for the next prompts
# fig tips reset &>/dev/null

# To update a specific variable:
# sed -i '' "s/FIG_VAR=.*/FIG_VAR=1/g" ~/.fig/user/config 2> /dev/null

# Check if onboarding variable is empty or not
# if [[ -z "$FIG_ONBOARDING" ]]; then
#   # Is empty. Set it to false
#   grep -q 'FIG_ONBOARDING' ~/.fig/user/config || echo "FIG_ONBOARDING=0" >> ~/.fig/user/config
# fi

# https://unix.stackexchange.com/questions/290146/multiple-logical-operators-a-b-c-and-syntax-error-near-unexpected-t
if  [[ "$FIG_ONBOARDING" == '0' ]] \
  && ([[ "$TERM_PROGRAM" == "iTerm.app" ]] \
    || [[ "$TERM_PROGRAM" == "Apple_Terminal" ]]); then
  # Check if we're logged in.
  if [[ "$FIG_LOGGED_IN" == '0' ]]; then
    # If we are actually logged in, update accordingly and run onboarding campaign.
    if [[ -n $(defaults read com.mschrage.fig userEmail 2> /dev/null) ]]; then
      sed -i '' "s/FIG_LOGGED_IN=.*/FIG_LOGGED_IN=1/g" ~/.fig/user/config 2> /dev/null
      if [[ -s ~/.fig/tools/drip/fig_onboarding.sh ]]; then
        ~/.fig/tools/drip/fig_onboarding.sh 
        exit
      fi
    fi
  else
    # If we are logged in, proceed as usual.
    if [[ -s ~/.fig/tools/drip/fig_onboarding.sh ]]; then
			  ~/.fig/tools/drip/fig_onboarding.sh
        exit
    fi
  fi
fi

if [[ "$FIG_LOGGED_IN" == "0" ]]; then
  exit
fi
# invariant:
# 1. User is logged in to Fig

export FIG_IS_RUNNING="$(fig app:running)"
# Ask for confirmation before updating
if [[ ! -z "$NEW_VERSION_AVAILABLE" ]]; then
  export NEW_VERSION_AVAILABLE="${NEW_VERSION_AVAILABLE}"
  export DISPLAYED_AUTOUPDATE_SETTINGS_HINT="${DISPLAYED_AUTOUPDATE_SETTINGS_HINT}"
  ~/.fig/tools/drip/prompt_to_update.sh
  unset NEW_VERSION_AVAILABLE
  unset DISPLAYED_AUTOUPDATE_SETTINGS_HINT
fi

if [[ -z "$APP_TERMINATED_BY_USER" && "$FIG_IS_RUNNING" == '0' ]]; then
  export DISPLAYED_AUTOLAUNCH_SETTINGS_HINT="${DISPLAYED_AUTOLAUNCH_SETTINGS_HINT}"
  ~/.fig/tools/drip/autolaunch.sh
  unset DISPLAYED_AUTOLAUNCH_SETTINGS_HINT
fi

# Show Fig tips
# Prevent termenv library from attempting to read color values and outputing random ANSI codes
# See https://github.com/muesli/termenv/blob/166cf3773788aab7e9bf5e34d8c0deb176b92bc8/termenv_unix.go#L172
TERM=screen fig tips prompt 2>/dev/null

unset FIG_IS_RUNNING

# NOTE: FIG_CHECKED_PROMPTS must be set to 1 in parent shell script.

# In the future we will calculate when a user signed up and if there are any
# drip campaigns remaining for the user. We will hardcode time since sign up
# versus drip campaign date here.
