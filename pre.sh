# fig_pty must be executable in path.
export FIG_INTEGRATION_VERSION=10
export TERM_SESSION_ID="$(uuidgen)"
if [ -t 1 ] && [ -x "$(command -v fig_pty)" ] && [ -n "${DISPLAY}" ]; then
  if [ -z "$FIG_TERM" ]; then
    fig_pty
  elif [ -z "$FIG_TERM_TMUX" ] && [ ! -z "$TMUX" ]; then
    fig_pty
  fi
fi
