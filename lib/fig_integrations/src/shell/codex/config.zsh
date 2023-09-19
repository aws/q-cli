
#--------------------------------------------------------------------#
# Global Configuration Variables                                     #
#--------------------------------------------------------------------#

# Color to use when highlighting suggestion
# Uses format of `region_highlight`
# More info: http://zsh.sourceforge.net/Doc/Release/Zsh-Line-Editor.html#Zle-Widgets
(( ! ${+FIG_AUTOSUGGEST_HIGHLIGHT_STYLE} )) &&
typeset -g FIG_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=8'

# Prefix to use when saving original versions of bound widgets
(( ! ${+FIG_AUTOSUGGEST_ORIGINAL_WIDGET_PREFIX} )) &&
typeset -g FIG_AUTOSUGGEST_ORIGINAL_WIDGET_PREFIX=autosuggest-orig-

# Strategies to use to fetch a suggestion
# Will try each strategy in order until a suggestion is returned
(( ! ${+FIG_AUTOSUGGEST_STRATEGY} )) && {
	typeset -ga FIG_AUTOSUGGEST_STRATEGY
	FIG_AUTOSUGGEST_STRATEGY=(codex)
}

# Widgets that clear the suggestion
(( ! ${+FIG_AUTOSUGGEST_CLEAR_WIDGETS} )) && {
	typeset -ga FIG_AUTOSUGGEST_CLEAR_WIDGETS
	FIG_AUTOSUGGEST_CLEAR_WIDGETS=(
		history-search-forward
		history-search-backward
		history-beginning-search-forward
		history-beginning-search-backward
		history-substring-search-up
		history-substring-search-down
		up-line-or-beginning-search
		down-line-or-beginning-search
		up-line-or-history
		down-line-or-history
		accept-line
		copy-earlier-word
	)
}

# Widgets that accept the entire suggestion
(( ! ${+FIG_AUTOSUGGEST_ACCEPT_WIDGETS} )) && {
	typeset -ga FIG_AUTOSUGGEST_ACCEPT_WIDGETS
	FIG_AUTOSUGGEST_ACCEPT_WIDGETS=(
		forward-char
		end-of-line
		vi-forward-char
		vi-end-of-line
		vi-add-eol
	)
}

# Widgets that accept the entire suggestion and execute it
(( ! ${+FIG_AUTOSUGGEST_EXECUTE_WIDGETS} )) && {
	typeset -ga FIG_AUTOSUGGEST_EXECUTE_WIDGETS
	FIG_AUTOSUGGEST_EXECUTE_WIDGETS=(
	)
}

# Widgets that accept the suggestion as far as the cursor moves
(( ! ${+FIG_AUTOSUGGEST_PARTIAL_ACCEPT_WIDGETS} )) && {
	typeset -ga FIG_AUTOSUGGEST_PARTIAL_ACCEPT_WIDGETS
	FIG_AUTOSUGGEST_PARTIAL_ACCEPT_WIDGETS=(
		forward-word
		emacs-forward-word
		vi-forward-word
		vi-forward-word-end
		vi-forward-blank-word
		vi-forward-blank-word-end
		vi-find-next-char
		vi-find-next-char-skip
	)
}

# Widgets that should be ignored (globbing supported but must be escaped)
(( ! ${+FIG_AUTOSUGGEST_IGNORE_WIDGETS} )) && {
	typeset -ga FIG_AUTOSUGGEST_IGNORE_WIDGETS
	FIG_AUTOSUGGEST_IGNORE_WIDGETS=(
		orig-\*
		beep
		run-help
		set-local-history
		which-command
		yank
		yank-pop
		zle-\*
	)
}

# Pty name for capturing completions for completion suggestion strategy
(( ! ${+FIG_AUTOSUGGEST_COMPLETIONS_PTY_NAME} )) &&
typeset -g FIG_AUTOSUGGEST_COMPLETIONS_PTY_NAME=fig_autosuggest_completion_pty
