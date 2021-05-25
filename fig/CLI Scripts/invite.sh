#!/usr/bin/env bash

# Fig onboarding shell script
# Based somewhat on oh my zshell https://github.com/ohmyzsh/ohmyzsh/blob/master/tools/install.sh

# Colors
BLACK=$(tput setaf 0)
RED=$(tput setaf 1)
GREEN=$(tput setaf 2)
YELLOW=$(tput setaf 3)
BLUE=$(tput setaf 4)
MAGENTA=$(tput setaf 5)
CYAN=$(tput setaf 6)
WHITE=$(tput setaf 7)

CODE=$(tput setaf 153)

# Other colors
LIME_YELLOW=$(tput setaf 190)
POWDER_BLUE=$(tput setaf 153)

# Weights and decoration
BOLD=$(tput bold)
UNDERLINE=$(tput smul)
UNDERLINE_END=$(tput rmul)
HIGHLIGHT=$(tput smso)
HIGHLIGHT_END=$(tput rmso)
NORMAL=$(tput sgr0)

# Structure
TAB='   '
SEPARATOR="  \n\n  --\n\n\n"


print_special() {
	echo "${TAB}$@${NORMAL}"$'\n'
}




# Get user's email address from defaults. send error to dev/null
mail=$(defaults read com.mschrage.fig userEmail 2> /dev/null)


if [[ "$mail" == "" ]]
then
	echo
    print_special "${BOLD}${RED}Error${NORMAL}${BOLD}: It does not seem like you are logged into Fig.${NORMAL}"
    echo
    print_special "Run ${BOLD}${MAGENTA}fig util:logout${NORMAL} then follow the prompts to log back. Then try again"
    exit 1
fi


# Given a user's email, get their referral code
url=$(curl "https://waitlist.withfig.com/waitlist/get-referral-link-from-email/$mail" 2> /dev/null )


if [[ "$url" == "" ]] || [[ "$url" == "ERROR" ]]
then
	echo
    print_special "${BOLD}${RED}Error${NORMAL}${BOLD}: We can't find a referral code for this email address: $mail${NORMAL}"
    echo
    print_special "${UNDERLINE}Are you sure you are logged in correctly?${UNDERLINE_END}"
    echo 
    print_special "Run ${BOLD}${MAGENTA}fig util:logout${NORMAL} then follow the prompts to log back. Then try again"
    echo
    print_special "If you think there is a mistake, please contact ${UNDERLINE}hello@fig.io${UNDERLINE_END}"
    echo
    echo
    print_special "P.S. This error may also occur if you are not connected to the internet"
    exit 1
else 
	echo $url | pbcopy
    echo
    echo "  ${BOLD}Thank you for sharing Fig.${NORMAL}"
    echo
    echo "› ${BOLD}${MAGENTA}${UNDERLINE}$url${UNDERLINE_END}${NORMAL}"
    echo "  Your referral link has been copied to the clipboard."
    echo
    
fi



