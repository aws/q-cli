echo "Deleting .fig folder & completion specs"
rm -rf ~/.fig

echo "Deleting WKWebViewCache"
fig util:reset-cache

# delete defaults
echo "Deleting fig defaults & preferences"
saved_id="$(defaults read com.mschrage.fig 'uuid')"
defaults delete com.mschrage.fig
defaults delete com.mschrage.fig.shared
defaults write com.mschrage.fig 'uuid' "$saved_id"

echo "Remove iTerm integration (if set up)"
rm ~/Library/Application\ Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.py

echo "Remove fish integration..."
rm ~/.config/fish/conf.d/fig.fish

# remove from .profiles
echo "Removing fig.sh setup from  .profile, .zprofile, .zshrc, .bash_profile, and .bashrc"

INSTALLATION1="#### FIG ENV VARIABLES ####"
INSTALLATION2="\[ -s ~/.fig/fig.sh \] && source ~/.fig/fig.sh"
INSTALLATION3="#### END FIG ENV VARIABLES ####"

sed -i '' -e "s/$INSTALLATION1//g" ~/.profile ~/.zprofile ~/.bash_profile ~/.bashrc ~/.zshrc
# change delimeter to '#' in order to escape '/'
sed -i '' -e "s#$INSTALLATION2##g" ~/.profile ~/.zprofile ~/.bash_profile ~/.bashrc ~/.zshrc
sed -i '' -e "s/$INSTALLATION3//g" ~/.profile ~/.zprofile ~/.bash_profile ~/.bashrc ~/.zshrc

#fig bg:event "Uninstall App"
echo "Finished removing fig resources. You may now delete the Fig app by moving it to the Trash."
#fig bg:alert "Done removing Fig resources." "You may now delete the Fig app by moving it to the Trash."
