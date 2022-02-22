# Source Mission Control Dotfiles
if [ -f ~/.fig/user/dotfiles/managed.zsh ]; then
  source ~/.fig/user/dotfiles/managed.zsh
fi

FIG_HOSTNAME=$(hostname -f 2> /dev/null || hostname)

if [[ -e /proc/1/cgroup ]] && grep -q docker /proc/1/cgroup; then
  FIG_IN_DOCKER=1
else
  FIG_IN_DOCKER=0
fi

__fig() {
    if [[ -d /Applications/Fig.app || -d ~/Applications/Fig.app ]] && command -v fig 2>&1 1>/dev/null; then
    fig "$@"
  fi
}

function fig_osc { printf "\033]697;"; printf $@; printf "\007"; }

FIG_HAS_ZSH_PTY_HOOKS=1
FIG_HAS_SET_PROMPT=0

fig_preexec() {
  # Restore user defined prompt before executing.
  [[ -v PS1 ]] && PS1="$FIG_USER_PS1"
  [[ -v PROMPT ]] && PROMPT="$FIG_USER_PROMPT"
  [[ -v prompt ]] && prompt="$FIG_USER_prompt"

  [[ -v PS2 ]] && PS2="$FIG_USER_PS2"
  [[ -v PROMPT2 ]] && PROMPT2="$FIG_USER_PROMPT2"

  [[ -v PS3 ]] && PS3="$FIG_USER_PS3"
  [[ -v PROMPT3 ]] && PROMPT3="$FIG_USER_PROMPT3"

  [[ -v PS4 ]] && PS4="$FIG_USER_PS4"
  [[ -v PROMPT4 ]] && PROMPT4="$FIG_USER_PROMPT4"

  [[ -v RPS1 ]] && RPS1="$FIG_USER_RPS1"
  [[ -v RPROMPT ]] && RPROMPT="$FIG_USER_RPROMPT"

  [[ -v RPS2 ]] && RPS2="$FIG_USER_RPS2"
  [[ -v RPROMPT2 ]] && RPROMPT2="$FIG_USER_RPROMPT2"

  FIG_HAS_SET_PROMPT=0
  fig_osc PreExec
}

fig_precmd() {
  local LAST_STATUS=$?

  fig_reset_hooks

  fig_osc "Dir=%s" "$PWD"
  fig_osc "Shell=zsh"
  fig_osc "PID=%d" "$$"
  fig_osc "SessionId=%s" "${TERM_SESSION_ID}"
  fig_osc "ExitCode=%s" "${LAST_STATUS}"
  fig_osc "TTY=%s" "${TTY}"
  fig_osc "Log=%s" "${FIG_LOG_LEVEL}"
  fig_osc "ZshAutosuggestionColor=%s" "${ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE}"

  if [[ -n "${SSH_TTY}" ]]; then
    fig_osc "SSH=1"
  else
    fig_osc "SSH=0"
  fi

  fig_osc "Docker=%d" "${FIG_IN_DOCKER}"
  fig_osc "Hostname=%s@%s" "${USER:-root}" "${FIG_HOSTNAME}"

  if [ $FIG_HAS_SET_PROMPT -eq 1 ]; then
    # ^C pressed while entering command, call preexec manually to clear fig prompts.
    fig_preexec
  fi

  START_PROMPT=$'\033]697;StartPrompt\007'
  END_PROMPT=$'\033]697;EndPrompt\007'
  NEW_CMD=$'\033]697;NewCmd\007'

  # Save user defined prompts.
  FIG_USER_PS1="$PS1"
  FIG_USER_PROMPT="$PROMPT"
  FIG_USER_prompt="$prompt"

  FIG_USER_PS2="$PS2"
  FIG_USER_PROMPT2="$PROMPT2"

  FIG_USER_PS3="$PS3"
  FIG_USER_PROMPT3="$PROMPT3"

  FIG_USER_PS4="$PS4"
  FIG_USER_PROMPT4="$PROMPT4"

  FIG_USER_RPS1="$RPS1"
  FIG_USER_RPROMPT="$RPROMPT"

  FIG_USER_RPS2="$RPS2"
  FIG_USER_RPROMPT2="$RPROMPT2"

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
  
  FIG_HAS_SET_PROMPT=1
}

fig_reset_hooks() {
  if [[ $precmd_functions[-1] != fig_precmd ]]; then
    precmd_functions=(${(@)precmd_functions:#fig_precmd} fig_precmd)
  fi
  if [[ $preexec_functions[1] != fig_preexec ]]; then
    preexec_functions=(fig_preexec ${(@)preexec_functions:#fig_preexec})
  fi
}

fig_reset_hooks
