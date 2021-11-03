#!/usr/bin/env bash

# Output helpers
RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
pass="${GREEN}pass${NC}"
fail="${RED}fail${NC}\n"
BOLD=$(tput bold)
NORMAL=$(tput sgr0)

function warn {
	echo -e "\n${YELLOW}$1${NC}"
}

function note {
	echo -e "\n${CYAN}$1${NC}"
}

function command {
	echo -e "${BLUE}$1${NC}"
}

function contact_support {
	echo -e "Run $(command "fig issue") to let us know about this error!"
	echo -e "Or, email us at ${CYAN}hello@fig.io${NC}!"
}

function fix {
	# Output the command we're running to a tmp file.
	# If the command exists in the file, then we've already
	# run this command and it's likely the fix didn't
	# work and we are in an infinite loop. We should
	# exit and cleanup the file.
	if grep -q "$*" fig_fixes &>/dev/null; then
		rm -f fig_fixes
		warn "\nLooks like we've already tried this fix before and it's not working.\n"
		echo "Run $(command "fig issue") to let us know!"
		echo "Or, email us at ${CYAN}hello@fig.io${NC}!"
		exit
	else
		echo "I can fix this!"
		echo -e "\n"
		echo "$*" >>fig_fixes
		echo "> $*"
		"$@"
		# There needs to be some time for any fig util scripts to do their
		# thing. 5 seconds seems to be sufficient.
		sleep 5
		echo -e "\n${GREEN}${BOLD}Fix applied!${NORMAL}${NC}\n"
		# Everytime we attempt a fix, there is a chance that other checks
		# will be affected. Script should be re-run to ensure we are
		# looking at an up to date environment.
		note "Let's restart our checks to see if the problem is resolved..."
		"$HOME/.fig/tools/$(basename "$0")" && exit
	fi
}

function is_installed {
	if mdfind "kMDItemKind == 'Application'" | grep -q "$1"; then
		echo true >/dev/null
	else
		echo false >/dev/null
	fi
}

# Initial checks
echo -e "\nLet's make sure Fig is running...\n"
if find "$HOME"/.fig/bin/fig >/dev/null; then
	echo -e "Fig bin exists: $pass"
else
	echo -e "Fig bin exists: $fail"
fi

if grep -q .fig/bin <<<"$PATH"; then
	echo -e "Fig in path: $pass"
else
	echo -e "Fig in path: $fail"
fi

# Is Fig running? Yes, continue. No, launch fig and restart doctor.
if [[ $("$HOME"/.fig/bin/fig app:running) == 1 ]]; then
	echo -e "Fig running: $pass"
	################
	# check .zshrc #
	################

	# Check that Fig ENV vars are all in the right place. If not, user needs to move them and put back
	# correctly.
	for file in "$HOME"/.profile "$HOME"/.zprofile "$HOME"/.bash_profile "$HOME"/.bashrc "$HOME"/.zshrc; do
		if [[ -f "$file" ]]; then
			# strip out whitespace and commented out lines
			clean_config=$(sed -e 's/#.*$//' "$file")
			# all lines that source other files or directly manipulate the $PATH
			path_manip=$(echo "$clean_config" | grep -E 'PATH|source')

			first=$(echo "$path_manip" | sed -n '1p')
			last=$(echo "$path_manip" | sed -n '$p')

			if [[ $first != "[ -s ~/.fig/shell/pre.sh ] && source ~/.fig/shell/pre.sh" ]] ||
				[[ $last != "[ -s ~/.fig/fig.sh ] && source ~/.fig/fig.sh" ]]; then
				warn "Fig ENV variables need to be at the very beginning and end of $file"
				warn "If you see the FIG ENV VARs in $file, make sure they're at the very beginning (pre) and end (post). Open a new terminal then rerun the the doctor."
				warn "If you don't see the FIG ENV VARs in $file, run 'fig util:install-script' to add them. Open a new terminal then rerun the doctor."
			fi
		fi
	done

	# Check for macOS version
	# Latest supported version is 10.13.6 (High Sierra)
	macos_version="$(sw_vers -productVersion)"
	IFS="=. " read -ra version <<<"$macos_version"
	major="${version[0]}"
	minor="${version[1]}"

	if (("$major" > 10)); then
		echo -e "macOS version: $pass"
	else
		if (("$major" == 10)); then
			if (("$minor" > 13)); then
				echo -e "macOS version: $pass"
			else
				echo -e "macOS version: $fail"
				warn "Your macOS version ($macos_version) is incompatible with Fig. Earliest supported version is 10.14.x (Mojave)"
			fi
		else
			echo -e "macOS version: $fail"
			warn "Your macOS version ($macos_version) is incompatible with Fig. Earliest supported version is 10.14.x (Mojave)"
		fi
	fi

	########################
	# check fig diagnostic #
	########################

	echo -e "\nLet's see what $(command "fig diagnostic") tells us...\n"
	# run fig diagnostic and split output into lines
	while IFS= read -ra line; do
		# for each line, split by ':' into the check and value
		IFS=: read -r check value <<<"${line[@]}"
		# strip leading and trailing whitespace from diagnostic value
		value=$(echo "$value" | xargs)

		case $check in
		"Installation Script")
			if [[ $value == true ]]; then
				echo -e "Installation script: $pass"
			else
				echo -e "Installation script: $fail"
				fix "$HOME"/.fig/bin/fig util:install-script
			fi
			;;
		"UserShell")
			if [[ $SHELL == *"zsh"* || $SHELL == *"bash"* || $SHELL == *"fish"* ]]; then
				echo -e "Compatible shell: $pass"
			else
				echo -e "Compatible shell: $fail"
				warn "You are not using a supported shell."
				echo -e "${CYAN}Only ${BOLD}zsh${NORMAL}${CYAN}, ${BOLD}bash${NORMAL}${CYAN}, or ${BOLD}fish${NORMAL}${CYAN} are integrated with Fig.${NC}"
				exit
			fi
			;;
		"Bundle path")
			if [[ $value == *"/Applications/Fig.app"* ]]; then
				echo -e "Bundle path: $pass"
			else
				echo -e "Bundle path: $fail"
				note "You need to install Fig in /Applications.\n"
				note "To fix: uninstall, then reinstall Fig."
				note "Remember to drag Fig into the Applications folder."
				exit
			fi
			;;
		"Autocomplete")
			if [[ $value == true ]]; then
				echo -e "Autocomplete enabled: $pass"
			else
				echo -e "Autocomplete enabled: $fail"
				note "To fix: run: $(command "fig settings autocomplete.disable false")"
				exit
			fi
			;;
		"CLI installed")
			if [[ $value == true ]]; then
				echo -e "CLI installed: $pass"
			else
				echo -e "CLI installed: $fail"
				fix "$HOME"/.fig/bin/fig util:symlink-cli
			fi
			;;
		"CLI tool path")
			if [[ $value == $HOME/.fig/bin/fig || $value == /usr/local/bin/.fig/bin/fig ]]; then
				echo -e "CLI tool path: $pass"
			else
				echo -e "CLI tool path: $fail"
				note "The Fig CLI must be in $HOME/.fig/bin/fig"
				exit
			fi
			;;
		"Accessibility")
			if [[ $value == true ]]; then
				echo -e "Accessibility enabled: $pass"
			else
				echo -e "Accessibility enabled: $fail"
				fix "$HOME"/.fig/bin/fig util:axprompt
			fi
			;;
		"SSH Integration")
			if [[ $value == true ]]; then
				if [ ! -w "$HOME"/.ssh/config ]; then
					note "FYI, your ssh config is read-only. Make sure Fig installed its integration in ~/.ssh/config.\n"
				fi

				if grep -q "Include ~/.fig/ssh" "$HOME"/.ssh/config; then
					echo -e "SSH config: $pass"
				else
					echo -e "SSH config: $fail"
					warn "SSH config is missing Include ~/.fig/ssh"
					note "To fix: Re-enable SSH integration from the Fig menu"
				fi
			fi
			;;
		"Tmux Integration")
			if [[ $value == true ]]; then
				echo -e "Tmux integration: $pass"
			else
				# Check if $HOME/.tmux.conf exists
				echo -e "Tmux integration: $fail"
				if ! find "$HOME"/.tmux.conf &>/dev/null; then
					warn "$HOME/.tmux.conf file is missing!"
					note "To fix: Re-enable Tmux integration from the Fig menu"
				else
					# Check if integration is in config
					if ! grep -q "source-file ~/.fig/tmux" "$HOME/.tmux.conf"; then
						warn "Missing 'source-file ~/.fig/tmux' in $HOME/.tmux.conf"
						note "To fix: Re-enable Tmux integration from the Fig menu"
					fi
				fi
			fi
			;;
		"iTerm Integration")
			if [[ $value == *"true"* ]]; then
				IFS="=. " read -ra version <<<"$(mdls -name kMDItemVersion /Applications/iTerm.app | xargs)"
				iterm_version="${version[1]}${version[2]}"
				if (("$iterm_version" > 33)); then
					echo -e "iTerm integration: $pass"
				else
					echo -e "iTerm integration: $fail"
					warn "Your iTerm version is incompatible with Fig. Please update iTerm to latest version."
				fi
			else
				# only care about integration if iTerm is installed
				if is_installed "iTerm.app"; then
					echo -e "iTerm integration: $fail"

					# API enabled?
					IFS="=; " read -r _ right <<<"$(defaults read com.googlecode.iterm2 | grep EnableAPIServer | xargs)"
					if [[ "$right" != 1 ]]; then
						warn "The iTerm API server is not enabled."
					fi

					# fig-iterm-integration.scpt exists in ~/Library/Application\ Support/iTerm2/Scripts/AutoLaunch/
					if ! find "$HOME"/Library/Application\ Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt &>/dev/null; then
						warn "fig-iterm-integration.scpt is missing."
					fi
				fi
			fi
			;;
		"Hyper Integration")
			if [[ $value == true ]]; then
				echo -e "Hyper integration: $pass"
			else
				# only care about integration if Hyper is installed
				if is_installed "Hyper.app"; then
					echo -e "Hyper integration: $fail"
					# Check ~/.hyper_plugins/local/fig-hyper-integration/index.js" exists
					if ! find "$HOME"/.hyper_plugins/local/fig-hyper-integration/index.js &>/dev/null; then
						warn "fig-hyper-integration plugin is missing!"
					fi
					# Tell them to add to localPlugins: ["fig-hyper-integration"]
					if ! grep -q "fig-hyper-integration" "$HOME"/.hyper.js; then
						warn "fig-hyper-integration plugin needs to be added to localPlugins!"
					fi
				fi
			fi
			;;
		"VSCode Integration")
			if [[ $value == true ]]; then
				echo -e "VSCode integration: $pass"
			else
				# only care about integration if VSCode is installed
				if is_installed "Visual Studio Code.app"; then
					echo -e "VSCode integration: $fail"
					# Check if withfig.fig exists
					if ! find "$HOME"/.vscode/extensions/withfig.fig-* &>/dev/null; then
						warn "VSCode extension is missing!"
					fi
				fi
			fi
			;;
		"Symlinked dotfiles")
			if [[ $value == true ]]; then
				dotfiles_symlinked=true
			else
				dotfiles_symlinked=false
			fi
			;;
		"PseudoTerminal Path")
			pseudo_terminal_path=$value
			;;
		"PATH")
			if [[ $value == "$pseudo_terminal_path" ]]; then
				echo -e "PATH and PseudoTerminal PATH match: $pass"
			else
				echo -e "PATH and PseudoTerminal PATH match: $fail"
				fix "$HOME"/.fig/bin/fig set:path
			fi
			;;
		"SecureKeyboardInput")
			secure_keyboard_input=$value
			;;
		"SecureKeyboardProcess")
			if [[ $secure_keyboard_input != true ]]; then
				echo -e "Secure keyboard input: $pass"
			else
				if is_installed "Bitwarden.app"; then
					IFS="=. " read -ra version <<<"$(mdls -name kMDItemVersion /Applications/Bitwarden.app | xargs)"
					bitwarden_version="${version[1]}${version[2]}"
					if (("$bitwarden_version" < 128)); then
						warn "Bitwarden may be enabling secure keyboard entry even when not focused."
						warn "This was fixed in version 1.28.0. See https://github.com/bitwarden/desktop/issues/991 for details."
						note "To fix: upgrade Bitwarden to the latest version."
						exit
					else
						echo -e "Secure keyboard input: $fail"
						warn "Secure keyboard input is on"
						warn "Secure keyboard process is $value"
						note "Please follow debugging steps at https://fig.io/docs/support/secure-keyboard-input"
						exit
					fi
				else
					echo -e "Secure keyboard input: $fail"
					warn "Secure keyboard input is on"
					warn "Secure keyboard process is $value"
					note "Please follow debugging steps at https://fig.io/docs/support/secure-keyboard-input"
					exit
				fi
			fi
			;;
		"FIG_INTEGRATION_VERSION")
			if [[ $value != *"?"* ]]; then
				echo -e "Fig integration version: $pass"
			else
				echo -e "Fig integration version: $fail"
				contact_support
				exit
			fi
			;;
		*)
			# Default pass
			continue
			;;
		esac
	done <<<"$("$HOME"/.fig/bin/fig diagnostic)"

	###############
	# misc checks #
	###############

	if [[ "$(defaults read com.mschrage.fig debugAutocomplete)" == 1 ]]; then
		warn "Forced popup is enabled.\nDisable in Fig menu under Integrations -> Developer -> Force Popup to Appear."
	fi

	###########################
	# additional help prompts #
	###########################

	# Clean up any fix logs
	rm -f fig_fixes

	if $dotfiles_symlinked; then
		note "FYI, looks like your dotfiles are symlinked."
		note "If you need to make modifications, make sure they're made in the right place."
	fi

	echo -e "\nFig still not working?"

	echo -e "Run $(command "fig issue") to let us know!"
	echo -e "Or, email us at ${CYAN}hello@fig.io${NC}!"
else
	echo -e "Fig running: $fail"
	fix "$HOME"/.fig/bin/fig launch
fi
