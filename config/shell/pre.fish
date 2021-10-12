contains $HOME/.fig/bin $fish_user_paths
or set -Ua fish_user_paths $HOME/.fig/bin

if [ -d /Applications/Fig.app -o -d ~/Applications/Fig.app ] \
  && command -v fig 1>/dev/null 2>/dev/null \
  && [ -t 1 ]

  # Generated automatically by iTerm and Terminal But needs to be
  # explicitly set for VSCode and Hyper. This variable is inherited when
  # new ttys are created using tmux and must be explictly overwritten.
  if [ -z "$TERM_SESSION_ID" ] || [ -n "$TMUX" ] || [ "$TERM_PROGRAM" = vscode ]
    export TERM_SESSION_ID=(uuidgen)
  end
  export FIG_INTEGRATION_VERSION=4

  if command -v figterm 1>/dev/null 2>/dev/null
    if [ -z "$FIG_TERM" ] || [ -z "$FIG_TERM_TMUX" -a -n "$TMUX" ]
      set FIG_SHELL (~/.fig/bin/fig_get_shell)
      set FIG_IS_LOGIN_SHELL 0
      if status --is-login
        set FIG_IS_LOGIN_SHELL 1
      end
      set FIG_TERM_NAME (basename "$FIG_SHELL")" (figterm)"
      set FIG_SHELL_PATH "$HOME/.fig/bin/$FIG_TERM_NAME"

      if [ -f "$FIG_SHELL_PATH" ]
        cp -p ~/.fig/bin/figterm "$FIG_SHELL_PATH"
      end
      
      exec bash -c "FIG_SHELL=$FIG_SHELL FIG_IS_LOGIN_SHELL=$FIG_IS_LOGIN_SHELL exec -a \"$FIG_TERM_NAME\" \"$FIG_SHELL_PATH\""
    end
  end
end

