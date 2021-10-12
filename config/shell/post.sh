if [[ -n "$BASH" ]]; then
  source ~/.fig/shell/post.bash
elif [[ -n "$ZSH_NAME" ]]; then
  source ~/.fig/shell/post.zsh
else
  # Fallback to naive method, exposed alteration of prompt variables if in unrecognized shell.
  if [ -z "${FIG_TERM_SHELL_VAR}" ]; then
    FIG_SHELL="\001\033]697;Shell=unknown\007\002"
    FIG_START_PROMPT="${FIG_SHELL}\001\033]697;StartPrompt\007\002"
    FIG_END_PROMPT="\001\033]697;EndPrompt\007\002"
    FIG_NEW_CMD="\001\033]697;NewCmd\007\002"

    PS1="${FIG_START_PROMPT}${PS1}${FIG_END_PROMPT}${FIG_NEW_CMD}"
    PS2="${FIG_START_PROMPT}${PS2}${FIG_END_PROMPT}"
    PS3="${FIG_START_PROMPT}${PS3}${FIG_END_PROMPT}${FIG_NEW_CMD}"

    FIG_TERM_SHELL_VAR=1
  fi
fi
