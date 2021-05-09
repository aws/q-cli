# Ideally would avoid this repetition with a function, but echoing/printf-ing
# the prompt can cause issues because of escape sequence conflict in prompt
# and echo/printf :(

if [ -n "$ZSH_VERSION" ]; then
  # Add %{brackets%} for zsh only.
  START_PROMPT=$'%{\e]697;START_PROMPT\e\\%}'
  END_PROMPT=$'%{\e]697;END_PROMPT\e\\%}'
  NEW_CMD=$'%{\e]697;NEW_CMD\e\\%}'

  PROMPT="$START_PROMPT${PROMPT}$END_PROMPT$NEW_CMD"
  PROMPT3="$START_PROMPT${PROMPT3}$END_PROMPT$NEW_CMD"

  PROMPT2="$START_PROMPT${PROMPT2}$END_PROMPT"
  RPS1="$START_PROMPT${RPS1}$END_PROMPT"
  RPROMPT="$START_PROMPT${RPROMPT}$END_PROMPT"
else
  START_PROMPT="\001\033]697;START_PROMPT\033\134\002"
  END_PROMPT="\001\033]697;END_PROMPT\033\134\002"
  NEW_CMD="\001\033]697;NEW_CMD\033\134\002"

  PS1="$START_PROMPT$PS1$END_PROMPT$NEW_CMD"
  PS3="$START_PROMPT$PS3$END_PROMPT$NEW_CMD"

  PS2="$START_PROMPT$PS2$END_PROMPT"
fi

if [ -n "$FIG_ENV" ]; then
  source $FIG_ENV
fi
