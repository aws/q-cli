mkdir -p ~/.fig/logs/
# Check if running under emulation to avoid running zsh specific code
# fixes https://github.com/withfig/fig/issues/291
EMULATION="$(emulate 2>>"$HOME"/.fig/logs/zsh.log)"
if [[ "${EMULATION}" != "zsh" ]]; then
  return
fi 

zmodload zsh/system

# Integrate with ZSH line editor
autoload -U +X add-zle-hook-widget
function fig_zsh_keybuffer() { 
  if (( PENDING || KEYS_QUEUED_COUNT )); then
    if (( ! ${+_fig_redraw_fd} )); then
      typeset -gi _fig_redraw_fd
      if sysopen -o cloexec -ru _fig_redraw_fd /dev/null; then
        zle -F $_fig_redraw_fd fig_zsh_redraw
      else
        unset _fig_redraw_fd
      fi
    fi
  else
    fig_zsh_redraw
  fi
 }

function fig_zsh_redraw() {
  if (( ${+_fig_redraw_fd} )); then
    zle -F "$_fig_redraw_fd"
    exec {_fig_redraw_fd}>&-
    unset _fig_redraw_fd
  fi

  # Redirect to /dev/null to avoid printing escape sequences
  fig hook editbuffer "${TERM_SESSION_ID}" "${FIG_INTEGRATION_VERSION}" "${TTY}" "$$" "${HISTNO}" "${CURSOR}" "$BUFFER" 2>&1 >/dev/null &!
}

function fig_hide() { 
  command -v fig 2>>"$HOME"/.fig/logs/zsh.log 1>/dev/null && fig hook hide 2>&1 1>/dev/null
}

# Hint: to list all special widgets, run `add-zle-hook-widget -L`

# Delete any widget, if it already exists
add-zle-hook-widget line-pre-redraw fig_zsh_keybuffer

# Workaround for zim crash
if [[ -z "$ZIM_HOME" ]]; then
  add-zle-hook-widget line-init fig_zsh_keybuffer
fi

# Hide when going through history (see also: histno logic in ShellHooksManager.updateKeybuffer)
add-zle-hook-widget history-line-set fig_hide

# Hide when searching
add-zle-hook-widget isearch-update fig_hide
