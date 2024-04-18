if [[ -n "$ZSH_NAME" ]]; then

# add ~/.local/bin to PATH
if [[ -d "${HOME}/.local/bin" ]] && [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
  PATH="${PATH:+"$PATH:"}${HOME}/.local/bin"
fi

if [[ -z "${TTY}" ]]; then
  TTY=$(tty)
fi
export TTY

export SHELL_PID="$$"

if [[ -z "${Q_SHELL}" ]]; then
  Q_SHELL=$(q _ get-shell)
fi

# shellcheck disable=SC2059
function fig_osc { printf "\033]697;$1\007" "${@:2}"; }

Q_HAS_SET_PROMPT=0

fig_preexec() {
  # Restore user defined prompt before executing.
  [[ -v PS1 ]] && PS1="$Q_USER_PS1"
  [[ -v PROMPT ]] && PROMPT="$Q_USER_PROMPT"
  [[ -v prompt ]] && prompt="$Q_USER_prompt"

  [[ -v PS2 ]] && PS2="$Q_USER_PS2"
  [[ -v PROMPT2 ]] && PROMPT2="$Q_USER_PROMPT2"

  [[ -v PS3 ]] && PS3="$Q_USER_PS3"
  [[ -v PROMPT3 ]] && PROMPT3="$Q_USER_PROMPT3"

  [[ -v PS4 ]] && PS4="$Q_USER_PS4"
  [[ -v PROMPT4 ]] && PROMPT4="$Q_USER_PROMPT4"

  [[ -v RPS1 ]] && RPS1="$Q_USER_RPS1"
  [[ -v RPROMPT ]] && RPROMPT="$Q_USER_RPROMPT"

  [[ -v RPS2 ]] && RPS2="$Q_USER_RPS2"
  [[ -v RPROMPT2 ]] && RPROMPT2="$Q_USER_RPROMPT2"

  Q_HAS_SET_PROMPT=0

  fig_osc "OSCLock=%s" "${QTERM_SESSION_ID}"
  fig_osc PreExec
}

fig_precmd() {
  local LAST_STATUS=$?

  fig_reset_hooks

  fig_osc "OSCUnlock=%s" "${QTERM_SESSION_ID}"
  fig_osc "Dir=%s" "$PWD"
  fig_osc "Shell=zsh"
  fig_osc "ShellPath=%s" "${Q_SHELL:-$SHELL}"
  if [[ -n "${WSL_DISTRO_NAME}" ]]; then
    fig_osc "WSLDistro=%s" "${WSL_DISTRO_NAME}"
  fi
  fig_osc "PID=%d" "$$"
  fig_osc "ExitCode=%s" "${LAST_STATUS}"
  fig_osc "TTY=%s" "${TTY}"
  fig_osc "Log=%s" "${Q_LOG_LEVEL}"
  fig_osc "ZshAutosuggestionColor=%s" "${ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE}"
  fig_osc "FigAutosuggestionColor=%s" "${Q_AUTOSUGGEST_HIGHLIGHT_STYLE}"
  fig_osc "User=%s" "${USER:-root}"

  if [ "$Q_HAS_SET_PROMPT" -eq 1 ]; then
    # ^C pressed while entering command, call preexec manually to clear fig prompts.
    fig_preexec
  fi

  START_PROMPT=$'\033]697;StartPrompt\007'
  END_PROMPT=$'\033]697;EndPrompt\007'
  NEW_CMD=$'\033]697;NewCmd='"${QTERM_SESSION_ID}"$'\007'

  # Save user defined prompts.
  Q_USER_PS1="$PS1"
  Q_USER_PROMPT="$PROMPT"
  Q_USER_prompt="$prompt"

  Q_USER_PS2="$PS2"
  Q_USER_PROMPT2="$PROMPT2"

  Q_USER_PS3="$PS3"
  Q_USER_PROMPT3="$PROMPT3"

  Q_USER_PS4="$PS4"
  Q_USER_PROMPT4="$PROMPT4"

  Q_USER_RPS1="$RPS1"
  Q_USER_RPROMPT="$RPROMPT"

  Q_USER_RPS2="$RPS2"
  Q_USER_RPROMPT2="$RPROMPT2"

  if [[ -v PROMPT ]]; then
    PROMPT="%{$START_PROMPT%}$PROMPT%{$END_PROMPT$NEW_CMD%}"
  elif [[ -v prompt ]]; then
    prompt="%{$START_PROMPT%}$prompt%{$END_PROMPT$NEW_CMD%}"
  else
    PS1="%{$START_PROMPT%}$PS1%{$END_PROMPT$NEW_CMD%}"
  fi

  if [[ -v PROMPT2 ]]; then
    PROMPT2="%{$START_PROMPT%}$PROMPT2%{$END_PROMPT%}"
  else
    PS2="%{$START_PROMPT%}$PS2%{$END_PROMPT%}"
  fi

  if [[ -v PROMPT3 ]]; then
    PROMPT3="%{$START_PROMPT%}$PROMPT3%{$END_PROMPT$NEW_CMD%}"
  else
    PS3="%{$START_PROMPT%}$PS3%{$END_PROMPT$NEW_CMD%}"
  fi

  if [[ -v PROMPT4 ]]; then
    PROMPT4="%{$START_PROMPT%}$PROMPT4%{$END_PROMPT%}"
  else
    PS4="%{$START_PROMPT%}$PS4%{$END_PROMPT%}"
  fi

  # Previously, the af-magic theme added a final % to expand. We need to paste without the %
  # to avoid doubling up and mangling the prompt. I've removed this workaround for now.
  if [[ -v RPROMPT ]]; then
    RPROMPT="%{$START_PROMPT%}$RPROMPT%{$END_PROMPT%}"
  else
    RPS1="%{$START_PROMPT%}$RPS1%{$END_PROMPT%}"
  fi

  if [[ -v RPROMPT2 ]]; then
    RPROMPT2="%{$START_PROMPT%}$RPROMPT2%{$END_PROMPT%}"
  else
    RPS2="%{$START_PROMPT%}$RPS2%{$END_PROMPT%}"
  fi

  Q_HAS_SET_PROMPT=1

  if command -v q >/dev/null 2>&1; then
    (command q _ pre-cmd --alias "$(\alias)" > /dev/null 2>&1 &) >/dev/null 2>&1
  fi
}

fig_reset_hooks() {
  # shellcheck disable=SC1087,SC2193
  if [[ "$precmd_functions[-1]" != fig_precmd ]]; then
    # shellcheck disable=SC2206,SC2296
    precmd_functions=(${(@)precmd_functions:#fig_precmd} fig_precmd)
  fi
  # shellcheck disable=SC1087,SC2193
  if [[ "$preexec_functions[1]" != fig_preexec ]]; then
    # shellcheck disable=SC2206,SC2296
    preexec_functions=(fig_preexec ${(@)preexec_functions:#fig_preexec})
  fi
}

fig_reset_hooks
if [[ -n "${PROCESS_LAUNCHED_BY_Q}" ]]; then
  fig_osc DoneSourcing
fi

fi

(command q _ pre-cmd --alias "$(\alias)" > /dev/null 2>&1 &) >/dev/null 2>&1
