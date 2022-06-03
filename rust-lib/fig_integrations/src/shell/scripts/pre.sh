#!/usr/bin/env bash

pathadd() {
  if [[ -d "$1" ]] && [[ ":$PATH:" != *":$1:"* ]]; then
    PATH="${PATH:+"$PATH:"}$1"
  fi
}

pathadd ~/.fig/bin
pathadd ~/.local/bin

if [[ -n "${FIG_NEW_SESSION}" ]]; then
  unset TERM_SESSION_ID
  unset FIG_TERM
  unset FIG_ENV_VAR
  unset FIG_NEW_SESSION
fi

# 0 = Yes, 1 = No, 2 = Fallback to FIG_TERM
fig _ should-figterm-launch 1>/dev/null 2>&1
SHOULD_FIGTERM_LAUNCH=$?

# Only launch figterm if current session is not already inside PTY and command exists.
# PWSH var is set when launched by `pwsh -Login`, in which case we don't want to init.
# It is not necessary in Fish.
if   [[ ! "${TERM_PROGRAM}" = WarpTerminal ]] \
  && [[ -z "${__PWSH_LOGIN_CHECKED}" ]] \
  && [[ -z "${INSIDE_EMACS}" ]] \
  && [[ "$__CFBundleIdentifier" != "com.vandyke.SecureCRT" ]] \
  && [[ -t 1 ]] \
  && [[ -z "${PROCESS_LAUNCHED_BY_FIG}" ]] \
  && [[ -z "${FIG_PTY}" ]] \
  && command -v figterm 1>/dev/null 2>&1 \
  && [[ "${SHOULD_FIGTERM_LAUNCH}" = 0 || "${SHOULD_FIGTERM_LAUNCH}" = 2 && (-z "${FIG_TERM}" || (-z "${FIG_TERM_TMUX}" && -n "${TMUX}")) ]]; then

  # Generated automatically by iTerm and Terminal, but needs to be
  # explicitly set for VSCode and Hyper. This variable is inherited when
  # new ttys are created using Tmux of VSCode and must be explictly
  # overwritten.
  if [[ -z "${TERM_SESSION_ID}" || -n "${TMUX}" ]]; then
    export TERM_SESSION_ID="$(uuidgen)"
  fi
  export FIG_INTEGRATION_VERSION=8
  # Pty module sets FIG_TERM or FIG_TERM_TMUX to avoid running twice.
  FIG_SHELL=$(fig _ get-shell)
  FIG_IS_LOGIN_SHELL="${FIG_IS_LOGIN_SHELL:='0'}"

  if ([[ -n "$BASH" ]] && shopt -q login_shell) \
    || [[ -n "$ZSH_NAME" && -o login ]]; then
    FIG_IS_LOGIN_SHELL=1
  fi

  # Do not launch figterm in non-interactive shells (like VSCode Tasks)
  if [[ $- == *i* ]]; then
    FIG_TERM_NAME="${FIG_SHELL} (figterm)"
    FIG_SHELL_PATH="$(command -v "$FIG_TERM_NAME" || echo "${HOME}/.fig/bin/$(basename "${FIG_SHELL}") (figterm)")"

    # Only copy figterm binary if it doesn't already exist
    if [[ ! -f "${FIG_SHELL_PATH}" ]]; then
      cp -p "$(command -v figterm)" "${FIG_SHELL_PATH}"
    fi

    FIG_EXECUTION_STRING="${BASH_EXECUTION_STRING:=$ZSH_EXECUTION_STRING}"

    # Get initial text.
    INITIAL_TEXT=""
    if [[ -z "${BASH}" || "${BASH_VERSINFO[0]}" -gt "3" ]]; then
      while read -t 0; do
        if [[ -n "${BASH}" ]]; then
          read -r
        fi
        INITIAL_TEXT="${INITIAL_TEXT}${REPLY}\n"
      done
    fi
    FIG_EXECUTION_STRING="${FIG_EXECUTION_STRING}" FIG_START_TEXT="$(printf "%b" "${INITIAL_TEXT}")" FIG_SHELL="${FIG_SHELL}" FIG_IS_LOGIN_SHELL="${FIG_IS_LOGIN_SHELL}" exec -a "${FIG_TERM_NAME}" "${FIG_SHELL_PATH}"
  fi
else
  FIG_DID_NOT_EXEC_FIGTERM=1
fi
