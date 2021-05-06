#MAGENTA=$(tput setaf 5)
#BOLD=$(tput bold)
#NORMAL=$(tput sgr0)
echo
echo "Pulling most up-to-date completion specs..."
echo "Run $(tput setaf 5)$(tput bold)fig docs$(tput sgr0) to learn how to contribute your own!"
echo
# Download all the files in the specs folder of this repo

base_url='https://waitlist.withfig.com/specs?version=';

# This is the current version of autocomplete
# It should be 1 ahead of the most recent branch we have created on github
# ie if we have a branch for v1 on withfig/autocomplete, make this v2
current_version=$(defaults read com.mschrage.fig "autocompleteVersion" 2>/dev/null) ;

mkdir -p ~/.fig/autocomplete; cd $_

curl -s "$base_url$current_version" | tar -xz --strip-components=1 specs
