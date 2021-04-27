# fig_pty must be executable in path.
if [ -x "$(command -v fig_pty)" ] && [ -n "${DISPLAY}" ]; then
  [ -z "${FIG_TERM}" ] && fig_pty
fi
