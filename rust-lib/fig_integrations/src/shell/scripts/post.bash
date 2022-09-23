if [[ -n "$BASH" ]]; then

pathadd() {
  if [[ -d "$1" ]] && [[ ":$PATH:" != *":$1:"* ]]; then
    PATH="${PATH:+"$PATH:"}$1"
  fi
}

pathadd ~/.fig/bin
pathadd ~/.local/bin

# Open workflows on keyboard shortcut
if [[ -z "${FIG_WORKFLOWS_KEYBIND}" ]]
then
  export FIG_WORKFLOWS_KEYBIND='^f'
fi

# we *would* install the keybind here, but the bash syntax is painful and we don't want to risk it
# if [[ "$(set -o | grep 'emacs\|\bvi\b' | cut -f2 | tr '\n' ':')" != 'off:off:' ]]; then
#   bind "\"${FIG_WORKFLOWS_KEYBIND}\":\"fig run\n\""
# fi

# if [[ "$FIG_DID_NOT_EXEC_FIGTERM" = 1 && "$FIG_TERM" != 1 ]] || [[ -n "${INSIDE_EMACS+x}" ]]; then
#   unset FIG_DID_NOT_EXEC_FIGTERM
#   return
# fi

TTY=$(tty)
export TTY
export FIG_PID="$$"

FIG_LAST_PS1="$PS1"
FIG_LAST_PS2="$PS2"
FIG_LAST_PS3="$PS3"

FIG_HOSTNAME=$(fig _ hostname || hostname -f 2> /dev/null || hostname)
FIG_SHELL_PATH=$(fig _ get-shell)

if [[ -e /proc/1/cgroup ]] && grep -q docker /proc/1/cgroup; then
  FIG_IN_DOCKER=1
elif [[ -f /.dockerenv ]]; then
  FIG_IN_DOCKER=1
else
  FIG_IN_DOCKER=0
fi

# Construct Operating System Command.
function fig_osc { printf "\033]697;%s\007" "$1" "${@:2}"; }

function __fig_preexec() {
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
  __bp_set_ret_value "${__fig_ret_value}" "${__bp_last_argument_prev_command:?}"
}

function __fig_pre_prompt () {
  __fig_ret_value="$?"

  if [[ -n "${SSH_TTY}" ]]; then
    fig_osc "SSH=1"
  else
    fig_osc "SSH=0"
  fi
  fig_osc "Docker=%d" "${FIG_IN_DOCKER}"
  fig_osc "Dir=%s" "${PWD}"
  fig_osc "Shell=bash"
  fig_osc "ShellPath=%s" "${FIG_SHELL_PATH:-$SHELL}"
  if [[ -n "${WSL_DISTRO_NAME}" ]]; then
    fig_osc "WSLDistro=%s" "${WSL_DISTRO_NAME}"
  fi
  fig_osc "PID=%d" "$$"
  fig_osc "SessionId=%s" "${TERM_SESSION_ID}"
  fig_osc "ExitCode=%s" "$__fig_ret_value"
  fig_osc "TTY=%s" "${TTY}"
  fig_osc "Log=%s" "${FIG_LOG_LEVEL}"
  fig_osc "Hostname=%s@%s" "${USER:-root}" "${FIG_HOSTNAME}"

  if command -v fig >/dev/null 2>&1; then
    case $(fig _ pre-cmd) in
      EXEC_NEW_SHELL)
        unset FIG_DOTFILES_SOURCED
        exec bash
        ;;
      *)
        ;;
    esac
  fi

  # Work around bug in CentOS 7.2 where preexec doesn't run if you press ^C
  # while entering a command.
  [[ -z "${_fig_done_preexec:-}" ]] && __fig_preexec ""
  _fig_done_preexec=""

  # Reset $?
  __bp_set_ret_value "${__fig_ret_value}" "${__bp_last_argument_prev_command}"
}

function __fig_post_prompt () {
  __fig_ret_value="$?"

  __fig_reset_hooks

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

__fig_reset_hooks() {
  # Rely on PROMPT_COMMAND instead of precmd_functions because precmd_functions
  # are all run before PROMPT_COMMAND.
  # Set PROMPT_COMMAND to "[
  #   __fig_pre_prompt,
  #   ...precmd_functions,
  #   ORIGINAL_PROMPT_COMMAND,
  #   __fig_post_prompt,
  #   __bp_interactive_mode
  # ]"
  local existing_prompt_command
  existing_prompt_command="${PROMPT_COMMAND}"
  existing_prompt_command="${existing_prompt_command//__fig_post_prompt[;$'\n']}"
  existing_prompt_command="${existing_prompt_command//__fig_post_prompt}"
  existing_prompt_command="${existing_prompt_command//__bp_interactive_mode[;$'\n']}"
  existing_prompt_command="${existing_prompt_command//__bp_interactive_mode}"
  __bp_sanitize_string existing_prompt_command "$existing_prompt_command"

  PROMPT_COMMAND=""
  if [[ -n "$existing_prompt_command" ]]; then
      PROMPT_COMMAND+=${existing_prompt_command}$'\n'
  fi;
  PROMPT_COMMAND+=$'__fig_post_prompt\n'
  PROMPT_COMMAND+='__bp_interactive_mode'

  if [[ ${precmd_functions[0]} != __fig_pre_prompt ]]; then
    for index in "${!precmd_functions[@]}"; do
      if [[ ${precmd_functions[$index]} == __fig_pre_prompt ]]; then
        unset -v 'precmd_functions[$index]'
      fi
    done
    precmd_functions=(__fig_pre_prompt "${precmd_functions[@]}")
  fi

  if [[ ${preexec_functions[0]} != __fig_preexec_preserve_status ]]; then
    for index in "${!preexec_functions[@]}"; do
      if [[ ${preexec_functions[$index]} == __fig_preexec_preserve_status ]]; then
        unset -v 'preexec_functions[$index]'
      fi
    done
    preexec_functions=(__fig_preexec_preserve_status "${preexec_functions[@]}")
  fi
}

# Ensure that bash-preexec is installed
# even if the user overrides COMMAND_PROMPT
# https://github.com/withfig/fig/issues/888
#
# We also need to ensure Warp is not running
# since they expect any plugins to not include
# it again
if [[ "${TERM_PROGRAM}" != "WarpTerminal" ]]; then
  __bp_install_after_session_init
fi
__fig_reset_hooks
if [[ -n "${PROCESS_LAUNCHED_BY_FIG}" ]]; then
  fig_osc DoneSourcing
fi

fi
