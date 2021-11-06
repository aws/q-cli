# Add preexec, but override __bp_adjust_histcontrol to preserve histcontrol.
source ~/.fig/shell/bash-preexec.sh
function __bp_adjust_histcontrol() { :; }

__fig() {
  if [[ -d /Applications/Fig.app || -d ~/Applications/Fig.app ]] && command -v fig 2>&1 1>/dev/null; then
    fig "$@"
  fi
}

FIG_LAST_PS1="$PS1"
FIG_LAST_PS2="$PS2"
FIG_LAST_PS3="$PS3"

FIG_HOSTNAME=$(hostname -f 2> /dev/null || hostname)

if [[ -e /proc/1/cgroup ]] && grep -q docker /proc/1/cgroup; then
  FIG_IN_DOCKER=1
else
  FIG_IN_DOCKER=0
fi

# Construct Operating System Command.
function fig_osc { printf "\033]697;"; printf $@; printf "\007"; }

function __fig_preexec() {
  __fig bg:exec $$ $TTY & disown

  fig_osc PreExec

  # Reset user prompts before executing a command, but only if it hasn't
  # changed since we last set it.
  if [[ -n "${FIG_USER_PS1+x}" && "${PS1}" = "${FIG_LAST_PS1}" ]]; then
    FIG_LAST_PS1="${FIG_USER_PS1}"
    export PS1="${FIG_USER_PS1}"
  fi
  if [[ -n "${FIG_USER_PS2+x}" && "${PS2}" = "${FIG_LAST_PS2}" ]]; then
    FIG_LAST_PS2="${FIG_USER_PS2}"
    export PS2="${FIG_USER_PS2}"
  fi
  if [[ -n "${FIG_USER_PS3+x}" && "${PS3}" = "${FIG_LAST_PS3}" ]]; then
    FIG_LAST_PS3="${FIG_USER_PS3}"
    export PS3="${FIG_USER_PS3}"
  fi

  _fig_done_preexec="yes"
}

function __fig_preexec_preserve_status() {
  __fig_ret_value="$?"
  __fig_preexec "$@"
  __bp_set_ret_value "${__fig_ret_value}" "${__bp_last_argument_prev_command}"
}

function __fig_prompt () {
  __fig_ret_value="$?"

  # Work around bug in CentOS 7.2 where preexec doesn't run if you press ^C
  # while entering a command.
  [[ -z "${_fig_done_preexec:-}" ]] && __fig_preexec ""
  _fig_done_preexec=""

  __fig bg:prompt $$ $TTY & disown

  # If FIG_USER_PSx is undefined or PSx changed by user, update FIG_USER_PSx.
  if [[ -z "${FIG_USER_PS1+x}" || "${PS1}" != "${FIG_LAST_PS1}" ]]; then
    FIG_USER_PS1="${PS1}"
  fi
  if [[ -z "${FIG_USER_PS2+x}" || "${PS2}" != "${FIG_LAST_PS2}" ]]; then
    FIG_USER_PS2="${PS2}"
  fi
  if [[ -z "${FIG_USER_PS3+x}" || "${PS3}" != "${FIG_LAST_PS3}" ]]; then
    FIG_USER_PS3="${PS3}"
  fi

  fig_osc "Dir=%s" "${PWD}"
  fig_osc "Shell=bash"
  fig_osc "PID=%d" "$$"
  fig_osc "SessionId=%s" "${TERM_SESSION_ID}"
  fig_osc "ExitCode=%s" "$__fig_ret_value"
  fig_osc "TTY=%s" "${TTY}"
  fig_osc "Log=%s" "${FIG_LOG_LEVEL}"

  if [[ -n "${SSH_TTY}" ]]; then
    fig_osc "SSH=1"
  else
    fig_osc "SSH=0"
  fi
  fig_osc "Docker=%d" "${FIG_IN_DOCKER}"
  fig_osc "Hostname=%s@%s" "${USER:-root}" "${FIG_HOSTNAME}"

  START_PROMPT="\[$(fig_osc StartPrompt)\]"
  END_PROMPT="\[$(fig_osc EndPrompt)\]"
  NEW_CMD="\[$(fig_osc NewCmd)\]"

  # Reset $? first in case it's used in $FIG_USER_PSx.
  __bp_set_ret_value "${__fig_ret_value}" "${__bp_last_argument_prev_command}"
  export PS1="${START_PROMPT}${FIG_USER_PS1}${END_PROMPT}${NEW_CMD}"
  export PS2="${START_PROMPT}${FIG_USER_PS2}${END_PROMPT}"
  export PS3="${START_PROMPT}${FIG_USER_PS3}${END_PROMPT}${NEW_CMD}"

  FIG_LAST_PS1="${PS1}"
  FIG_LAST_PS2="${PS2}"
  FIG_LAST_PS3="${PS3}"
}

# trap DEBUG -> preexec -> command -> PROMPT_COMMAND -> prompt shown.
preexec_functions=(__fig_preexec_preserve_status "${preexec_functions[@]}")
precmd_functions=(__fig_prompt "${precmd_functions[@]}")
