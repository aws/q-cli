make install

tmux new-session -s fig_dev -d "bash ${@}"
pid=$(tmux list-panes -t fig_dev -F '#{pane_pid}')

tmux split-window "tail -f out.${pid}.log"
tmux split-window -h "rm /tmp/fig.socket && nc -Ulk /tmp/fig.socket"
tmux attach -t fig_dev
