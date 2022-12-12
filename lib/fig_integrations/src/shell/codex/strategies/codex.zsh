
#--------------------------------------------------------------------#
# Codex Suggestion Strategy                                          #
#--------------------------------------------------------------------#
# Suggests the codex command.
#

_fig_autosuggest_strategy_codex() {
	typeset -g suggestion="$(command -v fig >/dev/null && fig _ codex --buffer ${BUFFER})"
}
