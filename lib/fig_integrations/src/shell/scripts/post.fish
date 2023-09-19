builtin contains $HOME/.fig/bin $PATH
or set --append PATH $HOME/.fig/bin

builtin contains $HOME/.local/bin $PATH
or set --append PATH $HOME/.local/bin

# if test "$FIG_DID_NOT_EXEC_FIGTERM" = 1
#     and test "$FIG_TERM" != 1
#     or set --query INSIDE_EMACS
#     set --erase FIG_DID_NOT_EXEC_FIGTERM
#     exit
# end


# Open scripts on keyboard shortcut
set --query FIG_SCRIPTS_KEYBIND; or set FIG_SCRIPTS_KEYBIND '\cf'

# function fig-open-scripts
#     fig run
#     commandline -f repaint
# end

# bind (string unescape $FIG_SCRIPTS_KEYBIND) fig-open-scripts

set --query TTY; or set TTY (command tty)
set --export TTY

set --export FIG_PID $fish_pid
set --export FIG_SET_PARENT $TERM_SESSION_ID
set --export LC_FIG_SET_PARENT $TERM_SESSION_ID

set --query FIG_SHELL_PATH; or set FIG_SHELL_PATH (fig _ get-shell)

function fig_osc
    builtin printf "\033]697;$argv[1]\007" $argv[2..-1]
end

function fig_copy_fn
    functions --erase $argv[2]
    functions --copy $argv[1] $argv[2]
    #builtin functions $argv[1] | sed "s/^function $argv[1]/function $argv[2]/" | source
end

function fig_fn_defined
    functions --query $argv[1]
    #test (builtin functions $argv[1] | command grep -vE '^ *(#|function |end$|$)' | command wc -l | command xargs) != 0
end

function fig_wrap_prompt
    set -l last_status $status
    fig_osc StartPrompt

    builtin printf "%b" (string join "\n" $argv)
    fig_osc EndPrompt

    return $last_status
end

function fig_preexec --on-event fish_preexec
    fig_osc "OSCLock=%s" "$FIGTERM_SESSION_ID"
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

    fig_osc "OSCUnlock=%s" "$FIGTERM_SESSION_ID"
    fig_osc "Dir=%s" "$PWD"
    fig_osc "Shell=fish"
    fig_osc "ShellPath=%s" "$FIG_SHELL_PATH"
    if test -n "$WSL_DISTRO_NAME"
        fig_osc "WSLDistro=%s" "$WSL_DISTRO_NAME"
    end
    fig_osc "PID=%d" "$fish_pid"
    fig_osc "ExitCode=%s" "$last_status"
    fig_osc "TTY=%s" "$TTY"
    fig_osc "Log=%s" "$FIG_LOG_LEVEL"
    fig_osc "FishSuggestionColor=%s" "$fish_color_autosuggestion"

    if test -n "$USER"
        fig_osc "User=%s" "$USER"
    else
        fig_osc "User=root" 
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
        fig_osc NewCmd=$FIGTERM_SESSION_ID
    end

    set fig_has_set_prompt 1

    if command -v fig &>/dev/null
        switch (fig _ pre-cmd)
            case EXEC_NEW_SHELL
                set -ge FIG_DOTFILES_SOURCED
                exec fish
        end
    end
end

set fig_has_set_prompt 0

if test -n "$PROCESS_LAUNCHED_BY_FIG"
    fig_osc DoneSourcing
end

begin; fig _ pre-cmd &> /dev/null &; end
