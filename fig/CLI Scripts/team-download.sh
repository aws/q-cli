#!/usr/bin/env bash

# This script lets users upload a specific cli spec to fig's cloud

MAGENTA=$(tput setaf 5)
RED=$(tput setaf 1)
BOLD=$(tput bold)
NORMAL=$(tput sgr0)
HIGHLIGHT=$(tput smso)
HIGHLIGHT_END=$(tput rmso)
TAB='   '

print_special() {
    echo "${TAB}$@${NORMAL}"$'\n'
}


#####################################
# State
#####################################

# Make sure dev_mode != 1 when this is pushed live
dev_mode=0

if [[ "$dev_mode" == '1' ]]
then
    echo
    echo "currently in dev mode"
    echo
    filename_endpoint="http://localhost:3000/autocomplete/team-file-name"
    download_endpoint="http://localhost:3000/autocomplete/download-team-file"

else
    filename_endpoint="https://waitlist.withfig.com/autocomplete/team-file-name"
    download_endpoint="https://waitlist.withfig.com/autocomplete/download-team-file"

fi

subcommand_name="team:download"
upload_subcommand_name="team:upload"

#####################################
# Functions
#####################################


prompt_to_logout() {

# cat <<EOF
    echo
    print_special "${BOLD}It looks like you are not properly logged into ${MAGENTA}Fig${NORMAL}"
    echo

    print_special "Please logout using ${BOLD}${MAGENTA}fig util:logout${NORMAL} then log back in and try again"

    # print_special "Fig will log you out and prompt you to log in again"
    # echo
    # print_special "When you've logged back in, please re-run ${BOLD}fig $subcommand_name ${NORMAL}"
    # echo
    # echo

    # # https://serverfault.com/questions/532559/bash-script-count-down-5-minutes-display-on-single-line
    # # Countdown timer
    
    # secs=$((8))

    # print_special "Press ctrl + c to cancel"
    # while [ $secs -gt 0 ]; do
    #     echo -ne "${TAB}Time remaining before logout: $secs\033[0K\r"
    #     sleep 1
    #     : $((secs--))
    # done

    # fig util:logout

    exit 1

}


#####################################
# Check token exists locally and is valid
#####################################

local_access_token=$(defaults read com.mschrage.fig access_token 2> /dev/null)

if [ -z $local_access_token ]
then
    prompt_to_logout
fi

#####################################
# Make post request to fig server
#####################################

file_name=$(curl -s -X POST \
-H "Authorization: Bearer $local_access_token" \
$filename_endpoint 2> /dev/null)



#####################################
# Support
#####################################

if [[ "$file_name" == ERROR* ]]
then

cat <<EOF

    ${BOLD}${RED}Error${NORMAL}

    $file_name

    There was an error downloading your team's private completion specs.

    Please contact ${BOLD}hello@fig.io${NORMAL} for support
EOF

elif [ -z $file_name ]
then

cat <<EOF

    There don't seem to be any private completion specs associated with your team's domain.
    
    Are you sure you / your team have uploaded private completion specs?

    
    --

    
    To upload completion specs, use:

    fig $upload_subcommand_name <file path to private completion spec>

    ${BOLD}Examples${NORMAL}

    fig $upload_subcommand_name ~/.fig/team/acme.js
    fig $upload_subcommand_name /path/to/acme.js

EOF

fi


# If we are here, we know we have a file that exists


# https://stackoverflow.com/questions/21950049/create-a-text-file-in-node-js-from-a-string-and-stream-it-in-response

# -o "$file_name"
result=$(curl -s -X POST \
-H "Authorization: Bearer $local_access_token" \
$download_endpoint \
2> /dev/null )

if [[ -z "$result" ]] || [[ "$result" == ERROR* ]]
then

    cat <<EOF

    ${BOLD}${MAGENTA}Error${NORMAL}

    $result

    There was an error downloading and/or saving your team's private autocomplete spec ${BOLD}${MAGENTA}$file_name${NORMAL}

    If this problem persists, please contact hello@fig.io for support.

EOF


else
    
    touch ~/.fig/team/$file_name
    echo "$result" > ~/.fig/team/$file_name

    # symlink and force option
    ln -fs ~/.fig/team/"$file_name" ~/.fig/autocomplete/"$file_name"
    
    cat <<EOF

    ${BOLD}${MAGENTA}Success${NORMAL}

    Your team's completion spec ${BOLD}${MAGENTA}$file_name${NORMAL} was successfully downloaded/updated.

EOF



fi


