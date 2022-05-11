contains $HOME/.fig/bin $fish_user_paths
or set -a PATH $HOME/.fig/bin

contains $HOME/.local/bin $fish_user_paths
or set -a PATH $HOME/.local/bin

if test "$FIG_DID_NOT_EXEC_FIGTERM" = 1
    set --erase FIG_DID_NOT_EXEC_FIGTERM
    exit
end

export TTY=(tty)
export FIG_PID=$fish_pid

set FIG_HOSTNAME (fig _ hostname; or hostname -f 2> /dev/null; or hostname)

if test -e /proc/1/cgroup; and grep -q docker /proc/1/cgroup
    set FIG_IN_DOCKER 1
else
    set FIG_IN_DOCKER 0
end

function fig_osc
    printf "\033]697;$argv[1]\007" $argv[2..-1]
end

function fig_copy_fn
    functions $argv[1] | sed "s/^function $argv[1]/function $argv[2]/" | source
end

function fig_fn_defined
    test (functions $argv[1] | grep -vE '^ *(#|function |end$|$)' | wc -l | xargs) != 0
end

function fig_wrap_prompt
    set -l last_status $status
    fig_osc StartPrompt

    printf "%b" (string join "\n" $argv)
    fig_osc EndPrompt

    return $last_status
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

    if test -n "$SSH_TTY"
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

    if test -n "$USER"
        fig_osc "Hostname=%s@%s" "$USER" "$FIG_HOSTNAME"
    else
        fig_osc "Hostname=%s@%s" root "$FIG_HOSTNAME"
    end

    if test $fig_has_set_prompt = 1
        fig_preexec
    end

    if fig_fn_defined fish_mode_prompt
        fig_copy_fn fish_mode_prompt fig_user_mode_prompt
        function fish_mode_prompt
            fig_wrap_prompt (fig_user_mode_prompt)
        end
    end

    if fig_fn_defined fish_right_prompt
        fig_copy_fn fish_right_prompt fig_user_right_prompt
        function fish_right_prompt
            fig_wrap_prompt (fig_user_right_prompt)
        end
    end

    fig_copy_fn fish_prompt fig_user_prompt
    function fish_prompt
        fig_wrap_prompt (fig_user_prompt)
        fig_osc NewCmd
    end

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

if test -n "$PROCESS_LAUNCHED_BY_FIG"
    fig_osc DoneSourcing
end
