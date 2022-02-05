#!/usr/bin/env bash

pathadd() {
  if [[ -d "$1" ]] && [[ ":$PATH:" != *":$1:"* ]]; then
    PATH="${PATH:+"$PATH:"}$1"
  fi
}

pathadd ~/.fig/bin

if [[ ! -z "${FIG_NEW_SESSION}" ]]; then
  unset TERM_SESSION_ID
  unset FIG_TERM
  unset FIG_ENV_VAR
  unset FIG_NEW_SESSION
fi

if [[ -n "$BASH" ]]; then
  # Add preexec, but override __bp_adjust_histcontrol to preserve histcontrol.
  source ~/.fig/shell/bash-preexec.sh
  function __bp_adjust_histcontrol() { :; }
fi

# Only launch figterm if current session is not already inside PTY and command exists
if   [[ ! "${TERM_PROGRAM}" = WarpTerminal ]] \
  && [[ -t 1 ]] \
  && [[ -z "${FIG_PTY}" ]] && command -v ~/.fig/bin/figterm 2>&1 1>/dev/null \
  && [[ -z "${FIG_TERM}" || (-z "${FIG_TERM_TMUX}" && -n "${TMUX}") ]]; then

  # Generated automatically by iTerm and Terminal, but needs to be
  # explicitly set for VSCode and Hyper. This variable is inherited when
  # new ttys are created using Tmux of VSCode and must be explictly
  # overwritten.

  if [[ -z "${TERM_SESSION_ID}" || -n "${TMUX}" ]]; then
    export TERM_SESSION_ID="$(uuidgen)"
  fi
  export FIG_INTEGRATION_VERSION=6
  # Pty module sets FIG_TERM or FIG_TERM_TMUX to avoid running twice. 
  FIG_SHELL=$(~/.fig/bin/fig_get_shell)
  FIG_IS_LOGIN_SHELL=0

  if ([[ -n "$BASH" ]] && shopt -q login_shell) \
    || ([[ -n "$ZSH_NAME" && -o login ]]); then
    FIG_IS_LOGIN_SHELL=1
  fi
  FIG_TERM_NAME="${FIG_SHELL} (figterm)"
  FIG_SHELL_PATH="${HOME}/.fig/bin/$(basename "${FIG_SHELL}") (figterm)"

  # Only copy figterm binary if it doesn't already exist
  if [[ ! -f "${FIG_SHELL_PATH}" ]]; then 
    cp -p ~/.fig/bin/figterm "${FIG_SHELL_PATH}"
  fi

  # Get initial text.
  INITIAL_TEXT=""
  if [[ -z "${BASH}" || "${BASH_VERSINFO[0]}" -gt "3" ]]; then
    while read -t 0; do
      if [[ -n "${BASH}" ]]; then
        read
      fi
      INITIAL_TEXT="${INITIAL_TEXT}${REPLY}\n"
    done
  fi
  
  FIG_START_TEXT="$(printf "%b" "${INITIAL_TEXT}")" FIG_SHELL="${FIG_SHELL}" FIG_IS_LOGIN_SHELL="${FIG_IS_LOGIN_SHELL}" exec -a "${FIG_TERM_NAME}" "${FIG_SHELL_PATH}"
fi
