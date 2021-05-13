export FIG_INTEGRATION_VERSION=10
export TERM_SESSION_ID=(uuidgen)

contains $HOME/.fig_pty/bin $fish_user_paths
or set -Ua fish_user_paths $HOME/.fig_pty/bin

if begin; and [ -t 1 ]; and test (command -v fig_pty); and [ -x (command -v fig_pty) ]; and [ -n "$DISPLAY" ]; end
  if [ -z "$FIG_TERM" ]
    fig_pty
  else if [ -z "$FIG_TERM_TMUX" -a -n "$TMUX" ]
    fig_pty
  end
end
