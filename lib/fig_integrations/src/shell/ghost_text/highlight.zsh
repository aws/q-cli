
#--------------------------------------------------------------------#
# Highlighting                                                       #
#--------------------------------------------------------------------#

# If there was a highlight, remove it
_fig_autosuggest_highlight_reset() {
	typeset -g _FIG_AUTOSUGGEST_LAST_HIGHLIGHT

	if [[ -n "$_FIG_AUTOSUGGEST_LAST_HIGHLIGHT" ]]; then
		region_highlight=("${(@)region_highlight:#$_FIG_AUTOSUGGEST_LAST_HIGHLIGHT}")
		unset _FIG_AUTOSUGGEST_LAST_HIGHLIGHT
	fi
}

# If there's a suggestion, highlight it
_fig_autosuggest_highlight_apply() {
	typeset -g _FIG_AUTOSUGGEST_LAST_HIGHLIGHT

	if (( $#POSTDISPLAY )); then
		typeset -g _FIG_AUTOSUGGEST_LAST_HIGHLIGHT="$#BUFFER $(($#BUFFER + $#POSTDISPLAY)) $FIG_AUTOSUGGEST_HIGHLIGHT_STYLE"
		region_highlight+=("$_FIG_AUTOSUGGEST_LAST_HIGHLIGHT")
	else
		unset _FIG_AUTOSUGGEST_LAST_HIGHLIGHT
	fi
}
