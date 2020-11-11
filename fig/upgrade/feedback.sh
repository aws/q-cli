#!/usr/bin/env bash

### Definitions

# Colors
MAGENTA=$(tput setaf 5)
RED=$(tput setaf 1)


# Weights and decoration
BOLD=$(tput bold)

NORMAL=$(tput sgr0)

# Structure
TAB='   '

print_special() {
	echo "${TAB}$@${NORMAL}"$'\n'
}


# Add colon to start to disable "illegal option -- []" logging
# Add colon after letter to show that option accepts argument
while getopts ":m:" OPTION
do
  case ${OPTION} in
    m) MSG="$OPTARG" ;;    
  esac
done

MSG=$(echo $MSG | xargs) # Trim whitespace


# If MSG was provided, hit endpoint
if [ -n "$MSG" ]
then

    EMAIL=$(defaults read com.mschrage.fig userEmail 2> /dev/null)

    if (curl -s --location --request POST 'https://fig-core-backend.herokuapp.com/feedback' \
    --header 'Content-Type: application/json' \
    --data-raw "{ \"message\": \"$MSG\", \"email\": \"$EMAIL\" }" > /dev/null)
    then    
        print_special "Feedback sent to ${MAGENTA}${BOLD}Fig${NORMAL} team. Thanks so much!"
    else
        print_special "Request to Fig server failed. Please try emailing hello@withfig.com"
    fi

else
    echo 
    print_special "Please provide a valid feedback message"
    print_special "e.g. ${MAGENTA}${BOLD}fig feedback -m '${NORMAL}I <3 fig${MAGENTA}${BOLD}'"

fi


 

# Returns email if valid, or empty string
# return_valid_email() {
#     regex="^(([-a-zA-Z0-9\!#\$%\&\'*+/=?^_`{\|}~]+|(\"([][,:;<>\&@a-zA-Z0-9\!#\$%\&\'*+/=?^_`{\|}~-]|(\\\\[\\ \"]))+\"))\.)*([-a-zA-Z0-9\!#\$%\&\'*+/=?^_`{\|}~]+|(\"([][,:;<>\&@a-zA-Z0-9\!#\$%\&\'*+/=?^_`{\|}~-]|(\\\\[\\ \"]))+\"))@\w((-|\w)*\w)*\.(\w((-|\w)*\w)*\.)*\w{2,4}$"

#     if [[ $i =~ $regex ]] ; then
#         echo "OK"
#     else
#         echo "not OK"
#     fi

# }

 
 
