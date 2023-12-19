if [[ -n "$BASH" ]]; then

# add ~/.local/bin to PATH
if [[ -d "${HOME}/.local/bin" ]] && [[ ":$PATH:" != *":${HOME}/.local/bin:"* ]]; then
  PATH="${PATH:+"$PATH:"}${HOME}/.local/bin"
fi

alias q='cw chat'

if [[ -z "${TTY}" ]]; then
  TTY=$(tty)
fi
export TTY

export SHELL_PID="$$"

CW_LAST_PS1="$PS1"
CW_LAST_PS2="$PS2"
CW_LAST_PS3="$PS3"

if [[ -z "${CW_SHELL}" ]]; then
  CW_SHELL=$(cw _ get-shell)
fi

# Construct Operating System Command.
# shellcheck disable=SC2059
function fig_osc { printf "\033]697;$1\007" "${@:2}"; }

function __fig_preexec() {
  fig_osc "OSCLock=%s" "${CWTERM_SESSION_ID}"
  fig_osc PreExec

  # Reset user prompts before executing a command, but only if it hasn't
  # changed since we last set it.
  if [[ -n "${CW_USER_PS1+x}" && "${PS1}" = "${CW_LAST_PS1}" ]]; then
    CW_LAST_PS1="${CW_USER_PS1}"
    export PS1="${CW_USER_PS1}"
  fi
  if [[ -n "${CW_USER_PS2+x}" && "${PS2}" = "${CW_LAST_PS2}" ]]; then
    CW_LAST_PS2="${CW_USER_PS2}"
    export PS2="${CW_USER_PS2}"
  fi
  if [[ -n "${CW_USER_PS3+x}" && "${PS3}" = "${CW_LAST_PS3}" ]]; then
    CW_LAST_PS3="${CW_USER_PS3}"
    export PS3="${CW_USER_PS3}"
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

  fig_osc "OSCUnlock=%s" "${CWTERM_SESSION_ID}"
  fig_osc "Dir=%s" "${PWD}"
  fig_osc "Shell=bash"
  fig_osc "ShellPath=%s" "${CW_SHELL:-$SHELL}"
  if [[ -n "${WSL_DISTRO_NAME}" ]]; then
    fig_osc "WSLDistro=%s" "${WSL_DISTRO_NAME}"
  fi
  fig_osc "PID=%d" "$$"
  fig_osc "ExitCode=%s" "$__fig_ret_value"
  fig_osc "TTY=%s" "${TTY}"
  fig_osc "Log=%s" "${CW_LOG_LEVEL}"
  fig_osc "User=%s" "${USER:-root}"

  if command -v cw >/dev/null 2>&1; then
    (command cw _ pre-cmd --alias "$(\alias)" > /dev/null 2>&1 &) >/dev/null 2>&1
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

  # If CW_USER_PSx is undefined or PSx changed by user, update CW_USER_PSx.
  if [[ -z "${CW_USER_PS1+x}" || "${PS1}" != "${CW_LAST_PS1}" ]]; then
    CW_USER_PS1="${PS1}"
  fi
  if [[ -z "${CW_USER_PS2+x}" || "${PS2}" != "${CW_LAST_PS2}" ]]; then
    CW_USER_PS2="${PS2}"
  fi
  if [[ -z "${CW_USER_PS3+x}" || "${PS3}" != "${CW_LAST_PS3}" ]]; then
    CW_USER_PS3="${PS3}"
  fi

  START_PROMPT="\[$(fig_osc StartPrompt)\]"
  END_PROMPT="\[$(fig_osc EndPrompt)\]"
  # shellcheck disable=SC2086
  # it's already double quoted, dummy
  NEW_CMD="\[$(fig_osc NewCmd=${CWTERM_SESSION_ID})\]"

  # Reset $? first in case it's used in $CW_USER_PSx.
  __bp_set_ret_value "${__fig_ret_value}" "${__bp_last_argument_prev_command}"
  export PS1="${START_PROMPT}${CW_USER_PS1}${END_PROMPT}${NEW_CMD}"
  export PS2="${START_PROMPT}${CW_USER_PS2}${END_PROMPT}"
  export PS3="${START_PROMPT}${CW_USER_PS3}${END_PROMPT}${NEW_CMD}"

  CW_LAST_PS1="${PS1}"
  CW_LAST_PS2="${PS2}"
  CW_LAST_PS3="${PS3}"
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
  # shellcheck disable=SC2128
  existing_prompt_command="${PROMPT_COMMAND}"
  existing_prompt_command="${existing_prompt_command//__fig_post_prompt[;$'\n']}"
  existing_prompt_command="${existing_prompt_command//__fig_post_prompt}"
  existing_prompt_command="${existing_prompt_command//__bp_interactive_mode[;$'\n']}"
  existing_prompt_command="${existing_prompt_command//__bp_interactive_mode}"
  __bp_sanitize_string existing_prompt_command "$existing_prompt_command"

  # shellcheck disable=SC2178
  PROMPT_COMMAND=""
  if [[ -n "$existing_prompt_command" ]]; then
        # shellcheck disable=SC2179
      PROMPT_COMMAND+=${existing_prompt_command}$'\n'
  fi;
  # shellcheck disable=SC2179
  PROMPT_COMMAND+=$'__fig_post_prompt\n'
  # shellcheck disable=SC2179
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
if [[ -n "${PROCESS_LAUNCHED_BY_CW}" ]]; then
  fig_osc DoneSourcing
fi

fi

(command cw _ pre-cmd --alias "$(\alias)" > /dev/null 2>&1 &) >/dev/null 2>&1
