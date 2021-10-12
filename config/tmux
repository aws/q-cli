set-hook -ga window-pane-changed 'run-shell "fig bg:tmux #{pane_id}"'
#set-hook -ga session-windows-changed 'run-shell "fig bg:tmux #{pane_id}"'
set-hook -ga client-session-changed 'run-shell "fig bg:tmux #{pane_id}"'

# set-hook -ga client-detached run-shell 'run-shell "fig bg:tmux closed"'
set-hook -ga session-closed 'run-shell "fig bg:tmux '%'"'