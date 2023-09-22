
#--------------------------------------------------------------------#
# GhostText Suggestion Strategy                                          #
#--------------------------------------------------------------------#
# Suggests the ghost_text command.
#

_fig_autosuggest_strategy_ghost_text() {
	typeset -g suggestion="$(command -v fig >/dev/null 2>&1 && cw _ ghost_text --buffer "${BUFFER}")"
}
