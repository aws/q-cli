
#--------------------------------------------------------------------#
# Codex Suggestion Strategy                                          #
#--------------------------------------------------------------------#
# Suggests the codex command.
#

_fig_autosuggest_strategy_codex() {
	typeset -g suggestion="$(fig _ codex --buffer ${BUFFER})"
}
