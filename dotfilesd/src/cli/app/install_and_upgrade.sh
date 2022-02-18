#!/usr/bin/env bash

# Install fig. Override if already exists
install_fig() {
  # Make files and folders that the user can edit (that aren't overridden by above)
  mkdir -p ~/.fig/bin ~/.fig/user/dotfiles ~/.fig/apps/

  # delete binary artifacts to ensure ad-hoc code signature works for arm64 binaries on M1
  rm ~/.fig/bin/{*figterm*,fig_get_shell,fig_callback,dotfilesd,fig}

  if [[ "$1" == "local" ]]; then
    cp -R "$PWD"/* ~/.fig
    cd ~/.fig
  fi

  BUNDLE="${FIG_BUNDLE_EXECUTABLES:-/Applications/Fig.app/Contents/MacOS/}"

  # rename figterm binaries to mirror supported shell
  # copy binaries on install to avoid issues with file permissions at runtime
  FIGTERM="${BUNDLE}/figterm" 

  cp -p "${FIGTERM}" "${HOME}"/.fig/bin/zsh\ \(figterm\)
  cp -p "${FIGTERM}" "${HOME}"/.fig/bin/bash\ \(figterm\)
  cp -p "${FIGTERM}" "${HOME}"/.fig/bin/fish\ \(figterm\)

  if [[ ! -f ~/.fig/settings.json ]]; then
    echo "{}" > ~/.fig/settings.json
  fi

  # Determine user's login shell by explicitly reading from "/Users/$(whoami)"
  # rather than ~ to handle rare cases where these are different.
  USER_SHELL="$(dscl . -read /Users/$(whoami) UserShell)"
  defaults write com.mschrage.fig userShell "${USER_SHELL}"

  USER_SHELL_TRIMMED="$(echo "${USER_SHELL}" | cut -d ' ' -f 2)"

  # Hardcode figcli path because symlinking has not happened when this script runs.
  FIG_CLI="${BUNDLE}/dotfilesd-darwin-universal"
  "${FIG_CLI}" settings userShell "${USER_SHELL_TRIMMED}"
  "${FIG_CLI}" install
}

install_fig

# Create config file if it doesn't exist.
if [[ ! -s ~/.fig/user/config ]]; then
  touch ~/.fig/user/config 
fi

add_conf_var() { grep -q "$1" ~/.fig/user/config || echo "$1=0" >> ~/.fig/user/config ; }

add_conf_var FIG_LOGGED_IN
add_conf_var FIG_ONBOARDING

TMUX_INTEGRATION=$'\n# Fig Tmux Integration: Enabled\nsource-file ~/.fig/tmux\n# End of Fig Tmux Integration'

# If ~/.tmux.conf.local exists, append integration here to workaround conflict with oh-my-tmux.
if [[ -s "${HOME}/.tmux.conf.local" ]]; then
  if ! grep -q 'source-file ~/.fig/tmux' ~/.tmux.conf.local; then 
    echo "${TMUX_INTEGRATION}" >> ~/.tmux.conf.local
  fi
elif [[ -s "${HOME}/.tmux.conf" ]]; then
  if ! grep -q 'source-file ~/.fig/tmux' ~/.tmux.conf; then 
    echo "${TMUX_INTEGRATION}" >> ~/.tmux.conf
  fi
fi

echo success
