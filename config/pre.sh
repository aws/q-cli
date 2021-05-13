pathadd() {
  if [ -d "$1" ] && [[ ":$PATH:" != *":$1:"* ]]; then
    PATH="${PATH:+"$PATH:"}$1"
  fi
}

pathadd ~/.fig_pty/bin

export TERM_SESSION_ID="$(uuidgen)"
export FIG_INTEGRATION_VERSION=2

if command -v fig_pty 1> /dev/null 2> /dev/null; then
  if [ -t 1 ] && [ -n "${DISPLAY}" ]; then
    if [ -z "$FIG_TERM" ]; then
      fig_pty
    elif [ -z "$FIG_TERM_TMUX" -a -n "$TMUX" ]; then
      fig_pty
    fi
  fi
fi
