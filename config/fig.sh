#!/usr/bin/env bash

pathadd() {
  if [[ -d "$1" ]] && [[ ":$PATH:" != *":$1:"* ]]; then
    PATH="${PATH:+"$PATH:"}$1"
  fi
}

__fig() {
  if [[ -d /Applications/Fig.app || -d ~/Applications/Fig.app ]] && command -v fig 2>&1 1>/dev/null; then
    fig "$@"
  fi
}

pathadd ~/.fig/bin

if [[ ! "${TERMINAL_EMULATOR}" = JetBrains-JediTerm ]]; then
  if [[ -t 1 ]] && [[ -z "${FIG_ENV_VAR}" || -n "${TMUX}" || "${TERM_PROGRAM}" = vscode ]]; then
    export FIG_ENV_VAR=1

    # Gives fig context for cwd in each window
    export TTY=$(tty)

    # Check for prompts or onboarding must be last, so Fig has context for
    # onboarding!
    if [[ -s ~/.fig/tools/prompts.sh ]]; then
       # don't source this, to ensure the #! is respected
       ~/.fig/tools/prompts.sh
       export FIG_CHECKED_PROMPTS=1
    fi
  fi

  source ~/.fig/shell/post.sh

  # todo: Add a check to confirm "add-zle-hook-widget" facility exists
  # Not included in fig.zsh, because should be run last
  if [[ -n "${ZSH_NAME}" ]]; then
    source ~/.fig/shell/zle.zsh
  fi
fi
