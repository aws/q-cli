#!/usr/bin/env bash

pathadd() {
  if [[ -d "$1" ]] && [[ ":$PATH:" != *":$1:"* ]]; then
    PATH="${PATH:+"$PATH:"}$1"
  fi
}

pathadd ~/.fig/bin

if [[ -t 1 ]] && [[ -z "${FIG_ENV_VAR}" || -n "${TMUX}" || "${TERM_PROGRAM}" = vscode ]]; then
  export FIG_ENV_VAR=1
  # Gives fig context for cwd in each window.
  export TTY=$(tty)

  # Onboard/prompts must be last, so we have context set up.
  if [[ -s ~/.fig/tools/prompts.sh ]]; then
     # Don't source this to avoid exiting on failure.
     ~/.fig/tools/prompts.sh
     export FIG_CHECKED_PROMPTS=1
  fi
fi

source ~/.fig/shell/post.sh

