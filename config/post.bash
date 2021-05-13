if [[ -z "$FIG_HAS_BASH_PTY_HOOKS" && "$-" == *i* && "$TERM" != linux ]]; then
FIG_HAS_BASH_PTY_HOOKS=1

FIG_LAST_PS1="$PS1"
FIG_LAST_PS2="$PS2"
FIG_LAST_PS3="$PS3"

[ -n "$PROMPT_COMMAND" ] && PROMPT_COMMAND+=$'\n'
PROMPT_COMMAND+='__fig_prompt'

# Add preexec, override __bp_adjust_histcontrol to preserve histcontrol.
source ~/.fig/bash-preexec.sh
function __bp_adjust_histcontrol() { :; }

# Construct Operating System Command.
function fig_osc { printf "\033]697;"; printf $@; printf "\007"; }

function __fig_preexec () {
  __fig_ret_value="$?"
  fig_osc PreExec

  # Reset user prompts before executing a command, but only if it hasn't changed
  # since we last set it.
  if [ -n "${FIG_USER_PS1+x}" -a "$PS1" = "$FIG_LAST_PS1" ]; then
    FIG_LAST_PS1="$FIG_USER_PS1"
    export PS1="$FIG_USER_PS1"
  fi
  if [ -n "${FIG_USER_PS2+x}" -a "$PS2" = "$FIG_LAST_PS2" ]; then
    FIG_LAST_PS2="$FIG_USER_PS2"
    export PS2="$FIG_USER_PS2"
  fi
  if [ -n "${FIG_USER_PS3+x}" -a "$PS3" = "$FIG_LAST_PS3" ]; then
    FIG_LAST_PS3="$FIG_USER_PS3"
    export PS3="$FIG_USER_PS3"
  fi

  _fig_done_preexec="yes"
  __bp_set_ret_value "$__fig_ret_value" "$__bp_last_argument_prev_command"
}

function __fig_prompt () {
  __fig_ret_value="$?"

  # Work around bug in CentOS 7.2 where preexec doesn't run if you press ^C while entering a command.
  [ -z "${_fig_done_preexec:-}" ] && __fig_preexec ""
  _fig_done_preexec=""

  # If FIG_USER_PSx is undefined or PSx changed by user, update FIG_USER_PSx.
  [ -z "${FIG_USER_PS1+x}" -o "$PS1" != "$FIG_LAST_PS1" ] && export FIG_USER_PS1="$PS1"
  [ -z "${FIG_USER_PS2+x}" -o "$PS2" != "$FIG_LAST_PS2" ] && export FIG_USER_PS2="$PS2"
  [ -z "${FIG_USER_PS3+x}" -o "$PS3" != "$FIG_LAST_PS3" ] && export FIG_USER_PS3="$PS3"

  fig_osc "Dir=%s" "$PWD"
  fig_osc "Shell=bash"
  START_PROMPT="\[$(fig_osc StartPrompt)\]"
  END_PROMPT="\[$(fig_osc EndPrompt)\]"
  NEW_CMD="\[$(fig_osc NewCmd)\]"

  # Reset $? first in case it's used in $FIG_USER_PSx.
  __bp_set_ret_value "$__fig_ret_value" "$__bp_last_argument_prev_command"
  export PS1="$START_PROMPT$FIG_USER_PS1$END_PROMPT$NEW_CMD"
  export PS2="$START_PROMPT$FIG_USER_PS2$END_PROMPT"
  export PS3="$START_PROMPT$FIG_USER_PS3$END_PROMPT$NEW_CMD"

  export FIG_LAST_PS1="$PS1"
  export FIG_LAST_PS2="$PS2"
  export FIG_LAST_PS3="$PS3"
}

# trap DEBUG -> preexec -> command -> PROMPT_COMMAND -> prompt shown.
preexec_functions+=(__fig_preexec)
fi
