contains $HOME/.fig/bin $fish_user_paths
or set -a PATH $HOME/.fig/bin

contains $HOME/.local/bin $fish_user_paths
or set -a PATH $HOME/.local/bin

if not test -z "$FIG_NEW_SESSION"
    set --erase TERM_SESSION_ID
    set --erase FIG_TERM
    set --erase FIG_ENV_VAR
    set --erase FIG_NEW_SESSION
end

if test "$TERM_PROGRAM" != WarpTerminal
    and test -z "$INSIDE_EMACS"
    and test "$__CFBundleIdentifier" != "com.vandyke.SecureCRT"
    and test -t 1
    and test -z "$PROCESS_LAUNCHED_BY_FIG"
    and test -z "$FIG_PTY"
    and command -v figterm 1>/dev/null 2>/dev/null
    and test -z "$FIG_TERM"
    or test -z "$FIG_TERM_TMUX" -a -n "$TMUX"

    # Generated automatically by iTerm and Terminal But needs to be
    # explicitly set for VSCode and Hyper. This variable is inherited when
    # new ttys are created using tmux and must be explictly overwritten.
    if test -z "$TERM_SESSION_ID"; or test -n "$TMUX"
        export TERM_SESSION_ID=(uuidgen)
    end
    export FIG_INTEGRATION_VERSION=8

    set FIG_SHELL (fig _ get-shell)
    set FIG_IS_LOGIN_SHELL 0
    if status --is-login
        set FIG_IS_LOGIN_SHELL 1
    end

    # Do not launch figterm in non-interactive shells (like VSCode Tasks)
    if status --is-interactive
        set FIG_TERM_NAME (basename "$FIG_SHELL")" (figterm)"
        set FIG_SHELL_PATH (command -v "$FIG_TERM_NAME" || echo "$HOME/.fig/bin/$FIG_TERM_NAME")

        # Only copy figterm binary if it doesn't already exist
        # WARNING: copying file if it already exists results
        # in crashes. See https://github.com/withfig/fig/issues/548
        if not test -f "$FIG_SHELL_PATH"
            cp -p ~/.fig/bin/figterm "$FIG_SHELL_PATH"
        end

        # Need to exec bash because we're using 'exec -a <name>'
        # to set argv[0] and fish's exec doesn't have this option
        exec bash -c "FIG_SHELL=$FIG_SHELL FIG_IS_LOGIN_SHELL=$FIG_IS_LOGIN_SHELL exec -a \"$FIG_TERM_NAME\" \"$FIG_SHELL_PATH\""
    end
# else
#     set -g FIG_DID_NOT_EXEC_FIGTERM 1
end
