command mkdir -p ~/.local/bin >/dev/null

builtin contains $HOME/.local/bin $PATH
or set --append PATH $HOME/.local/bin

builtin contains $HOME/.local/bin $PATH
or set --append PATH $HOME/.local/bin

if not test -z "$FIG_NEW_SESSION"
    set --erase CWTERM_SESSION_ID
    set --erase CW_TERM
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

if test -z "$SHOULD_CWTERM_LAUNCH"
    # 0 = Yes, 1 = No, 2 = Fallback to CW_TERM
    cw _ should-figterm-launch 1>/dev/null 2>&1
    set SHOULD_CWTERM_LAUNCH $status
end

if test -t 1
    and test -z "$PROCESS_LAUNCHED_BY_FIG"
    and command -v cwterm 1>/dev/null 2>/dev/null
    and test "$SHOULD_CWTERM_LAUNCH" -eq 0 -o \( "$SHOULD_CWTERM_LAUNCH" -eq 2 -a \( -z "$CW_TERM" -o \( -z "$CW_TERM_TMUX" -a -n "$TMUX" \) \) \)

    if test -z "$CW_SHELL"
        set CW_SHELL (cw _ get-shell)
    end
    set FIG_IS_LOGIN_SHELL 0
    if status --is-login
        set FIG_IS_LOGIN_SHELL 1
    end

    # Do not launch cwterm in non-interactive shells (like VSCode Tasks)
    if status --is-interactive
        set CW_TERM_NAME (command basename "$CW_SHELL")" (cwterm)"
        if test -x "$HOME/.local/bin/$CW_TERM_NAME"
            set CW_TERM_PATH "$HOME/.local/bin/$CW_TERM_NAME"
        else
            set CW_TERM_PATH (command -v cwterm || echo "$HOME/.local/bin/cwterm")
        end

        # Need to exec bash because we're using 'exec -a <name>'
        # to set argv[0] and fish's exec doesn't have this option
        exec bash -c "FIG_PARENT=$FIG_PARENT CW_SHELL=$CW_SHELL FIG_IS_LOGIN_SHELL=$FIG_IS_LOGIN_SHELL exec -a \"$CW_TERM_NAME\" \"$CW_TERM_PATH\""
    end
    # else
    #     set -g FIG_DID_NOT_EXEC_FIGTERM 1
end
