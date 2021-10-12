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
    fig_endpoint="http://localhost:3000/autocomplete/upload-team-file"
else
    fig_endpoint="https://waitlist.withfig.com/autocomplete/upload-team-file"
fi

subcommand_name="team:upload"

#####################################
# Functions
#####################################

#### Log out of Fig ####
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
# Check file path exists and is valid
#####################################

if [ -z $1 ]
then

cat <<EOF

    Please include an absolute or relative file path to your team's completion spec.

    fig $subcommand_name <file path to completion spec>

    ${BOLD}Examples${NORMAL}

    fig $subcommand_name ~/.fig/team/acme.js
    fig $subcommand_name /path/to/acme.js

EOF
exit 1

# check if file path exists
elif [ ! -f $1 ]
then

cat <<EOF

    File path ${BOLD}$1${NORMAL} does not exist. Please include a valid file path to your team's completion spec

    fig $subcommand_name <file path to completion spec>

    ${BOLD}Examples${NORMAL}

    fig $subcommand_name ~/.fig/team/acme.js
    fig $subcommand_name /path/to/acme.js

EOF

exit 1

fi



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

result=$(curl -s -X POST \
-H "Authorization: Bearer $local_access_token" \
-H "file_path: $1" \
-H "Content-Type: text/plain" \
--data-binary "@$1"  \
$fig_endpoint 2> /dev/null )



#####################################
# Support
#####################################

if [[ -z "$result" ]] || [[ "$result" == ERROR* ]]
then

cat <<EOF

    ${BOLD}${RED}Error${NORMAL}

    $result

    There was an error uploading your completion spec.

    Please contact ${BOLD}hello@fig.io${NORMAL} for support
EOF


else

cat <<EOF

    ${BOLD}${MAGENTA}Success${NORMAL}

    Your completion spec was successfully uploaded to your team's private repo.

    Tell your team to run ${BOLD}fig team:download${NORMAL} to install/update your private completion specs

EOF

fi
