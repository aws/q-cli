#!/usr/bin/env bash

mkdir -p "${HOME}/.local/bin" > /dev/null 2>&1

# add ~/.local/bin to PATH
if [[ -d "${HOME}/.local/bin" ]] && [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
  PATH="${PATH:+"$PATH:"}${HOME}/.local/bin"
fi

if [[ -n "${FIG_NEW_SESSION}" ]]; then
  unset CWTERM_SESSION_ID
  unset CW_TERM
  unset FIG_ENV_VAR
  unset FIG_NEW_SESSION
fi

if [[ -z "${FIG_SET_PARENT_CHECK}" ]]; then
  # Load parent from env variables
  if [[ "$FIG_SET_PARENT" = "" && "$LC_FIG_SET_PARENT" != "" ]]; then
    export FIG_SET_PARENT=$LC_FIG_SET_PARENT
    unset -v LC_FIG_SET_PARENT
  fi
  if [[ "$FIG_PARENT" = "" && "$FIG_SET_PARENT" != "" ]]; then
    export FIG_PARENT=$FIG_SET_PARENT
    unset -v FIG_SET_PARENT
  fi
  export FIG_SET_PARENT_CHECK=1
fi

# 0 = Yes, 1 = No, 2 = Fallback to CW_TERM
if [ -z "${SHOULD_CWTERM_LAUNCH}" ]; then
  cw _ should-figterm-launch 1>/dev/null 2>&1
  SHOULD_CWTERM_LAUNCH=$?
fi

# Only launch figterm if current session is not already inside PTY and command exists.
# PWSH var is set when launched by `pwsh -Login`, in which case we don't want to init.
# It is not necessary in Fish.
if   [[ -t 1 ]] \
  && [[ -z "${PROCESS_LAUNCHED_BY_FIG}" ]] \
  && command -v cwterm 1>/dev/null 2>&1 \
  && [[ ("${SHOULD_CWTERM_LAUNCH}" -eq 0) || (("${SHOULD_CWTERM_LAUNCH}" -eq 2) && (-z "${CW_TERM}" || (-z "${CW_TERM_TMUX}" && -n "${TMUX}"))) ]]
then
  # Pty module sets CW_TERM or CW_TERM_TMUX to avoid running twice.
  if [ -z "${CW_SHELL}" ]; then
    CW_SHELL=$(cw _ get-shell)
  fi
  FIG_IS_LOGIN_SHELL="${FIG_IS_LOGIN_SHELL:='0'}"

  # shellcheck disable=SC2030
  if ([[ -n "$BASH" ]] && shopt -q login_shell) \
    || [[ -n "$ZSH_NAME" && -o login ]]; then
    FIG_IS_LOGIN_SHELL=1
  fi

  # Do not launch figterm in non-interactive shells (like VSCode Tasks)
  if [[ $- == *i* ]]; then
    CW_TERM_NAME="$(basename "${CW_SHELL}") (cwterm)"
    if [[ -x "${HOME}/.local/bin/${CW_TERM_NAME}" ]]; then
      CW_TERM_PATH="${HOME}/.local/bin/${CW_TERM_NAME}"
    else
      CW_TERM_PATH="$(command -v cwterm || echo "${HOME}/.local/bin/cwterm")"
    fi

    FIG_EXECUTION_STRING="${BASH_EXECUTION_STRING:=$ZSH_EXECUTION_STRING}"

    # Get initial text.
    INITIAL_TEXT=""
    # shellcheck disable=SC2031
    if [[ -z "${BASH}" || "${BASH_VERSINFO[0]}" -gt "3" ]]; then
      while read -rt 0; do
        if [[ -n "${BASH}" ]]; then
          read -r
        fi
        INITIAL_TEXT="${INITIAL_TEXT}${REPLY}\n"
      done
    fi
    FIG_EXECUTION_STRING="${FIG_EXECUTION_STRING}" FIG_START_TEXT="$(printf "%b" "${INITIAL_TEXT}")" CW_SHELL="${CW_SHELL}" FIG_IS_LOGIN_SHELL="${FIG_IS_LOGIN_SHELL}" exec -a "${CW_TERM_NAME}" "${CW_TERM_PATH}"
  fi
# else
#   FIG_DID_NOT_EXEC_FIGTERM=1
fi
