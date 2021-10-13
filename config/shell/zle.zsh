
# Check if running under emulation to avoid running zsh specific code
# fixes https://github.com/withfig/fig/issues/291
EMULATION="$(emulate 2>"$HOME"/.fig/logs/zsh.log)"
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


  (echo fig bg:zsh-keybuffer "${TERM_SESSION_ID}" "${FIG_INTEGRATION_VERSION}" "${TTY}" "$$" "${HISTNO}" "${CURSOR}" \""$BUFFER"\" | /usr/bin/base64 | /usr/bin/nc -U /tmp/fig.socket 2>"$HOME"/.fig/logs/zsh.log &)
}

function fig_hide() { 
  command -v fig 2>"$HOME"/.fig/logs/zsh.log 1>/dev/null && fig bg:hide &!
}

# Hint: to list all special widgets, run `add-zle-hook-widget -L`

# Delete any widget, if it already exists
add-zle-hook-widget line-pre-redraw fig_zsh_keybuffer

add-zle-hook-widget line-init fig_zsh_keybuffer

# Hide when going through history (see also: histno logic in ShellHooksManager.updateKeybuffer)
add-zle-hook-widget history-line-set fig_hide

# Hide when searching
add-zle-hook-widget isearch-update fig_hide

# Create insertion facility
function fig_insert () {
  immediate=$(< ~/.fig/zle/immediate)
  insertion=$(< ~/.fig/zle/insert)
  deletion=$(< ~/.fig/zle/delete)
  offset=$(< ~/.fig/zle/offset)

  if [ ! $deletion = "0" ]; then
    LBUFFER=${LBUFFER:0:-deletion}
  fi

  RBUFFER=${insertion}${RBUFFER}
  CURSOR=$CURSOR+${#insertion}-$offset

  if [ $immediate = "1" ]; then
    zle accept-line
  fi

}

zle -N fig_insert

# Used to force z-asug to clear suggestion
# This function will by used when inserting immediately
function fig_insert_and_clear_autosuggestion () {
  zle fig_insert
}

zle -N fig_insert_and_clear_autosuggestion


# Store bindkey command to reset original 'main' keymap
RESET_KEYMAP=$(bindkey -lL main)
# Note: ‘bindkey -lL main’ shows which keymap is linked to ‘main’, if any,
# and hence if the standard emacs or vi emulation is in effect.
# https://zsh.sourceforge.io/Doc/Release/Zsh-Line-Editor.html#index-bindkey

# Bind to arbitrary unicode character
# If this changes, make sure to update coresponding keycode in ZLEIntegration.insert
# And increment $FIG_INTEGRATION_VERSION

# bind to viins keymap
bindkey -v '◧' fig_insert
bindkey -v '◨' fig_insert_and_clear_autosuggestion

# bind to emacs keymap
bindkey -e '◧' fig_insert
bindkey -e '◨' fig_insert_and_clear_autosuggestion

# Restore original keymapm (the -v and -e flags override the main keymap)
eval $RESET_KEYMAP

# Ensure compatibility w/ z-asug -- resolves https://github.com/withfig/fig/issues/62
# Add `fig_insert_and_clear_autosuggestion` to list of widgets that clear suggestions, if not already included
if ! command test -z "$ZSH_AUTOSUGGEST_CLEAR_WIDGETS" && ! (($ZSH_AUTOSUGGEST_CLEAR_WIDGETS[(Ie)fig_insert_and_clear_autosuggestion])); then
  ZSH_AUTOSUGGEST_CLEAR_WIDGETS=(${(@)ZSH_AUTOSUGGEST_CLEAR_WIDGETS} fig_insert_and_clear_autosuggestion)
fi

