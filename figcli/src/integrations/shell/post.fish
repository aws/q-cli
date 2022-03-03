contains $HOME/.fig/bin $fish_user_paths
or set -Ua fish_user_paths $HOME/.fig/bin

contains $HOME/.local/bin $fish_user_paths
or set -Ua fish_user_paths $HOME/.local/bin

export TTY=(tty)

set FIG_HOSTNAME (hostname -f 2> /dev/null || hostname)

if [ -e /proc/1/cgroup ] && grep -q docker /proc/1/cgroup
  set FIG_IN_DOCKER 1
else
  set FIG_IN_DOCKER 0
end

function fig_osc; printf "\033]697;"; printf $argv; printf "\007"; end
function fig_copy_fn; functions $argv[1] | sed "s/^function $argv[1]/function $argv[2]/" | source; end
function fig_fn_defined; test (functions $argv[1] | grep -vE '^ *(#|function |end$|$)' | wc -l | xargs) != 0; end

function fig_wrap_prompt
  set -l last_status $status
  fig_osc StartPrompt

  sh -c "exit $last_status"
  printf "%b" (string join "\n" $argv)
  fig_osc EndPrompt

  sh -c "exit $last_status"
end

function fig_preexec --on-event fish_preexec
  fig_osc PreExec

  if fig_fn_defined fig_user_mode_prompt
    fig_copy_fn fig_user_mode_prompt fish_mode_prompt
  end

  if fig_fn_defined fig_user_right_prompt
    fig_copy_fn fig_user_right_prompt fish_right_prompt
  end

  fig_copy_fn fig_user_prompt fish_prompt

  set fig_has_set_prompt 0
end

function fig_precmd --on-event fish_prompt
  set -l last_status $status

  if [ -n "$SSH_TTY" ]
    fig_osc "SSH=1"
  else
    fig_osc "SSH=0"
  end

  fig_osc "Docker=%d" "$FIG_IN_DOCKER"
  fig_osc "Dir=%s" "$PWD"
  fig_osc "Shell=fish"
  fig_osc "PID=%d" "$fish_pid"
  fig_osc "SessionId=%s" "$TERM_SESSION_ID"
  fig_osc "ExitCode=%s" "$last_status"
  fig_osc "TTY=%s" (tty)
  fig_osc "Log=%s" "$FIG_LOG_LEVEL"
  fig_osc "FishSuggestionColor=%s" "$fish_color_autosuggestion"

  if [ -n "$USER" ]
    fig_osc "Hostname=%s@%s" "$USER" "$FIG_HOSTNAME"
  else
    fig_osc "Hostname=%s@%s" "root" "$FIG_HOSTNAME"
  end

  if [ $fig_has_set_prompt = 1 ]
    fig_preexec
  end

  if fig_fn_defined fish_mode_prompt
    fig_copy_fn fish_mode_prompt fig_user_mode_prompt
    function fish_mode_prompt; fig_wrap_prompt (fig_user_mode_prompt); end
  end

  if fig_fn_defined fish_right_prompt
    fig_copy_fn fish_right_prompt fig_user_right_prompt
    function fish_right_prompt; fig_wrap_prompt (fig_user_right_prompt); end
  end

  fig_copy_fn fish_prompt fig_user_prompt
  function fish_prompt; fig_wrap_prompt (fig_user_prompt); fig_osc NewCmd; end

  set fig_has_set_prompt 1

  # Check if we have a new dotfiles to load
  if command -v fig &>/dev/null
    if fig _ prompt-dotfiles-changed
      set -ge FIG_DOTFILES_SOURCED
      exec fish
    end
  end
end

set fig_has_set_prompt 0
