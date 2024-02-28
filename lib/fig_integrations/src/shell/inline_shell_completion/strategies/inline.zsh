
#--------------------------------------------------------------------#
# InlineShell Suggestion Strategy                                          #
#--------------------------------------------------------------------#
# Suggests the inline_shell_completion command.
#

_cw_autosuggest_strategy_inline_shell_completion() {
	typeset -g suggestion="$(command -v cw >/dev/null 2>&1 && cw _ inline-shell-completion --buffer "${BUFFER}")"
}
