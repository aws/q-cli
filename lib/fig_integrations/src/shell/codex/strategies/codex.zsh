
#--------------------------------------------------------------------#
# Codex Suggestion Strategy                                          #
#--------------------------------------------------------------------#
# Suggests the codex command.
#

_fig_autosuggest_strategy_codex() {
	typeset -g suggestion="$(command -v fig >/dev/null 2>&1 && fig _ codex --buffer "${BUFFER}")"
}
