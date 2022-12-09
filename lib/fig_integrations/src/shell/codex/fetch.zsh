
#--------------------------------------------------------------------#
# Fetch Suggestion                                                   #
#--------------------------------------------------------------------#
# Loops through all specified strategies and returns a suggestion
# from the first strategy to provide one.
#

_fig_autosuggest_fetch_suggestion() {
	typeset -g suggestion
	local -a strategies
	local strategy

	# Ensure we are working with an array
	strategies=(${=FIG_AUTOSUGGEST_STRATEGY})

	for strategy in $strategies; do
		# Try to get a suggestion from this strategy
		_fig_autosuggest_strategy_$strategy "$1"

		# Ensure the suggestion matches the prefix
		[[ "$suggestion" != "$1"* ]] && unset suggestion

		# Break once we've found a valid suggestion
		[[ -n "$suggestion" ]] && break
	done
}
