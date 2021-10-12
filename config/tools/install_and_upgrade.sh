#!/usr/bin/env bash
#set -e

# This is the fig installation script. It runs just after you sign in for the
# first time.

# Replace TAG_NAME with the commit hash, git tag (like v1.0.25), or leave empty
# This script should be run via curl:
#   sh <(curl -fsSL https://raw.githubusercontent.com/withfig/config/main/tools/install_and_upgrade.sh) TAG_NAME
# or via wget:
#   sh <(wget -qO- https://raw.githubusercontent.com/withfig/config/main/tools/install_and_upgrade.sh) TAG_NAME
# or via fetch:
#   sh <(fetch -o - https://raw.githubusercontent.com/withfig/config/main/tools/install_and_upgrade.sh) TAG_NAME

FIGREPO='https://github.com/withfig/config.git'

# We are constantly pushing changes to the public repo.  Each version of the
# swift app is only compatible with a certain version of the public repo.
# The commit hash is passed in as a parameter to this script.  We hard reset to
# this commit hash. If we don't get a hash, we just hard reset to the most
# recent version of the repo...
FIG_TAG="$1"
if [[ -z "${FIG_TAG}" ]]; then
  FIG_TAG="main"
fi

echo "Tag is ${FIG_TAG}"
date_string=$(date '+%Y-%m-%d_%H-%M-%S')

error() {
  echo "Error: $@" >&2
  exit 1
}

# Install fig. Override if already exists
install_fig() {
  # Create fig dir an cd into it
  mkdir -p ~/.fig

  # delete binary artifacts to ensure ad-hoc code signature works for arm64 binaries on M1
  rm ~/.fig/bin/*figterm*
  rm ~/.fig/bin/fig_callback
  rm ~/.fig/bin/fig_get_shell

  if [[ "${FIG_TAG}" == "local" ]]; then
    cp -R "$PWD"/* ~/.fig
    cd ~/.fig
  else
    cd ~/.fig
    curl "https://codeload.github.com/withfig/config/tar.gz/${FIG_TAG}" \
      | tar -xz --strip-components=1 \
      || (
        echo "downloading from main instead of fig tag_name" \
          && curl https://codeload.github.com/withfig/config/tar.gz/main \
            | tar -xz --strip-components=1 \
        ) \
      || error "pulling withfig/config repo failed"
  fi

  # Make files and folders that the user can edit (that aren't overridden by above)
  mkdir -p ~/.fig/{bin,zle,autocomplete}
  mkdir -p ~/.fig/user/{aliases,apps,autocomplete,aliases}

  # rename figterm binaries to mirror supported shell
  # copy binaries on install to avoid issues with file permissions at runtime
  cp -p "${HOME}"/.fig/bin/figterm "${HOME}"/.fig/bin/zsh\ \(figterm\)
  cp -p "${HOME}"/.fig/bin/figterm "${HOME}"/.fig/bin/bash\ \(figterm\)
  cp -p "${HOME}"/.fig/bin/figterm "${HOME}"/.fig/bin/fish\ \(figterm\)

  if [[ ! -f ~/.fig/settings.json ]]; then
    echo "{}" > ~/.fig/settings.json
  fi

  touch ~/.fig/user/aliases/_myaliases.sh

  # Figpath definition.
  touch ~/.fig/user/figpath.sh

  # Determine user's login shell by explicitly reading from "/Users/$(whoami)"
  # rather than ~ to handle rare cases where these are different.
  USER_SHELL="$(dscl . -read /Users/$(whoami) UserShell)"
  defaults write com.mschrage.fig userShell "${USER_SHELL}"

  USER_SHELL_TRIMMED="$(echo "${USER_SHELL}" | cut -d ' ' -f 2)"

  # Hardcode figcli path because symlinking has not happened when this script
  # runs.
  FIGCLI=/Applications/Fig.app/Contents/MacOS/figcli 
  "${FIGCLI}" settings userShell "${USER_SHELL_TRIMMED}"
  
  "${FIGCLI}" settings pty.path $("${USER_SHELL_TRIMMED}" -li -c "/usr/bin/env | /usr/bin/grep '^PATH=' | /bin/cat | /usr/bin/sed 's|PATH=||g'") 

  # hotfix for infinite looping when writing "☑ fig" title to a tty backed by figterm
  "${FIGCLI}" settings autocomplete.addStatusToTerminalTitle false

  # Restart file watcher
  "${FIGCLI}" settings:init

  # Define the figpath variable in the figpath file
  # The file should look like this:
  #   export FIGPATH="~/.fig/bin:~/run:"
  #   FIGPATH=$FIGPATH'~/abc/de fg/hi''~/zyx/wvut'
  if ! grep -q 'FIGPATH=$FIGPATH' ~/.fig/user/figpath.sh; then 
    echo $'\n''FIGPATH=$FIGPATH' >> ~/.fig/user/figpath.sh
  fi
}

setup_onboarding() {
  # Create config file if it doesn't exist.
  if [[ ! -s ~/.fig/user/config ]]; then
    touch ~/.fig/user/config 
  fi

  # If this is first download, mark download time as now.
  grep -q 'DOWNLOAD_TIME' ~/.fig/user/config || echo "DOWNLOAD_TIME=$(date +'%s')" >> ~/.fig/user/config

  # Create last_update if it doesn't exist and mark last update as now.
  grep -q 'LAST_UPDATE' ~/.fig/user/config || echo "LAST_UPDATE=$(date +'%s')" >> ~/.fig/user/config
  sed -i '' "s/LAST_UPDATE=.*/LAST_UPDATE=$(date +'%s')/g" ~/.fig/user/config 2> /dev/null

  add_conf_var() { grep -q "$1" ~/.fig/user/config || echo "$1=0" >> ~/.fig/user/config ; }

  add_conf_var FIG_LOGGED_IN
  add_conf_var FIG_ONBOARDING
  add_conf_var DONT_SHOW_DRIP
  for num in ONE TWO THREE FOUR FIVE SIX SEVEN; do
    add_conf_var "DRIP_${num}"
  done
}

# hotfix for infinite looping when writing "☑ fig" title to a tty backed by figterm
disable_setting_tty_title() {
  defaults write com.mschrage.fig addIndicatorToTitlebar false
}

install_fig
setup_onboarding
disable_setting_tty_title

~/.fig/tools/install_integrations.sh

echo success
