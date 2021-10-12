autoload -Uz add-zsh-hook

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
  __fig bg:exec $$ $TTY &!

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
  __fig bg:prompt $$ $TTY &!

  if [ $FIG_HAS_SET_PROMPT -eq 1 ]; then
    # ^C pressed while entering command, call preexec manually to clear fig prompts.
    fig_preexec
  fi

  fig_osc "Dir=%s" "$PWD"
  fig_osc "Shell=zsh"
  fig_osc "PID=%d" "$$"
  fig_osc "SessionId=%s" "${TERM_SESSION_ID}"
  fig_osc "ExitCode=%s" "${LAST_STATUS}"
  fig_osc "TTY=%s" "${TTY}"
  fig_osc "Log=%s" "${FIG_LOG_LEVEL}"

  if [[ -n "${SSH_TTY}" ]]; then
    fig_osc "SSH=1"
  else
    fig_osc "SSH=0"
  fi

  fig_osc "Docker=%d" "${FIG_IN_DOCKER}"
  fig_osc "Hostname=%s@%s" "${USER:-root}" "${FIG_HOSTNAME}"

  START_PROMPT=$(fig_osc StartPrompt)
  END_PROMPT=$(fig_osc EndPrompt)
  NEW_CMD=$(fig_osc NewCmd)

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

  [[ -v PS1 ]] && PS1="%{$START_PROMPT%}$PS1%{$END_PROMPT$NEW_CMD%}"
  [[ -v PROMPT ]] && PROMPT="%{$START_PROMPT%}$PROMPT%{$END_PROMPT$NEW_CMD%}"
  [[ -v prompt ]] && prompt="%{$START_PROMPT%}$prompt%{$END_PROMPT$NEW_CMD%}"

  [[ -v PS2 ]] && PS2="%{$START_PROMPT%}$PS2%{$END_PROMPT%}"
  [[ -v PROMPT2 ]] && PROMPT2="%{$START_PROMPT%}$PROMPT2%{$END_PROMPT%}"

  [[ -v PS3 ]] && PS3="%{$START_PROMPT%}$PS3%{$END_PROMPT$NEW_CMD%}"
  [[ -v PROMPT3 ]] && PROMPT3="%{$START_PROMPT%}$PROMPT3%{$END_PROMPT$NEW_CMD%}"

  [[ -v PS4 ]] && PS4="%{$START_PROMPT%}$PS4%{$END_PROMPT%}"
  [[ -v PROMPT4 ]] && PROMPT4="%{$START_PROMPT%}$PROMPT4%{$END_PROMPT%}"

  # The af-magic theme adds a final % to expand. We need to paste without the %
  # to avoid doubling up and mangling the prompt.
  if [[ "$ZSH_THEME" == "af-magic" ]]; then
    RPS1="%{$START_PROMPT%}$RPS1{$END_PROMPT%}"
  else
    [[ -v RPS1 ]] && RPS1="%{$START_PROMPT%}$RPS1%{$END_PROMPT%}"
  fi
  [[ -v RPROMPT ]] && RPROMPT="%{$START_PROMPT%}$RPROMPT%{$END_PROMPT%}"

  [[ -v RPS2 ]] && RPS2="%{$START_PROMPT%}$RPS2%{$END_PROMPT%}"
  [[ -v RPROMPT2 ]] && RPROMPT2="%{$START_PROMPT%}$RPROMPT2%{$END_PROMPT%}"
  
  FIG_HAS_SET_PROMPT=1

  # Temporary workaround for bug where istrip is activated when running brew install.
  # When istrip is turned on, input characters are strippped to seven bits.
  # This causes zle insertion to stop due to our reliance on `fig_insert` being bound to a unicode character
  [[ -t 1 ]] && command stty -istrip
}

add-zsh-hook precmd fig_precmd
add-zsh-hook preexec fig_preexec
