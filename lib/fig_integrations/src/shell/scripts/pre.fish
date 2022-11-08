command mkdir -p ~/.fig/bin >/dev/null

builtin contains $HOME/.fig/bin $PATH
or set --append PATH $HOME/.fig/bin

builtin contains $HOME/.local/bin $PATH
or set --append PATH $HOME/.local/bin

if not test -z "$FIG_NEW_SESSION"
    set --erase FIGTERM_SESSION_ID
    set --erase FIG_TERM
    set --erase FIG_ENV_VAR
    set --erase FIG_NEW_SESSION
end

# Load parent from env variables
if test -n "$FIG_SET_PARENT"; and test -z "$LC_FIG_SET_PARENT"
    set --export FIG_SET_PARENT $LC_FIG_SET_PARENT
end
if test -n "$FIG_PARENT"; and test -z "$FIG_SET_PARENT"
    set --export FIG_PARENT $FIG_SET_PARENT
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

    set FIG_SHELL (fig _ get-shell)
    set FIG_IS_LOGIN_SHELL 0
    if status --is-login
        set FIG_IS_LOGIN_SHELL 1
    end

    # Do not launch figterm in non-interactive shells (like VSCode Tasks)
    if status --is-interactive
        set FIG_TERM_NAME figterm
        set FIG_SHELL_PATH (command -v "$FIG_TERM_NAME" || echo "$HOME/.fig/bin/figterm")

        # Need to exec bash because we're using 'exec -a <name>'
        # to set argv[0] and fish's exec doesn't have this option
        exec bash -c "FIG_PARENT=$FIG_PARENT FIG_SHELL=$FIG_SHELL FIG_IS_LOGIN_SHELL=$FIG_IS_LOGIN_SHELL exec -a \"$FIG_TERM_NAME\" \"$FIG_SHELL_PATH\""
    end
    # else
    #     set -g FIG_DID_NOT_EXEC_FIGTERM 1
end
