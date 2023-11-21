
#--------------------------------------------------------------------#
# GhostText Suggestion Strategy                                          #
#--------------------------------------------------------------------#
# Suggests the ghost_text command.
#

_cw_autosuggest_strategy_ghost_text() {
	typeset -g suggestion="$(command -v cw >/dev/null 2>&1 && cw _ ghost-text --buffer "${BUFFER}")"
}
