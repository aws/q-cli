echo "Deleting .fig folder & completion specs"
rm -rf ~/.fig

echo "Delete backup Fig CLI"
rm /usr/local/bin/fig

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
rm ~/.config/iterm2/AppSupport/Scripts/AutoLaunch/fig-iterm-integration.py
rm ~/Library/Application\ Support/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.scpt

echo "Remove VSCode integration (if set up)"
rm -rf ~/.vscode/extensions/withfig.fig-*
rm -rf ~/.vscode-insiders/extensions/withfig.fig-*

echo "Remove fish integration..."
rm ~/.config/fish/conf.d/fig.fish

# remove from .profiles
echo "Removing fig.sh setup from  .profile, .zprofile, .zshrc, .bash_profile, and .bashrc"

INSTALLATION1="#### FIG ENV VARIABLES ####"
INSTALLATION2="\[ -s ~/.fig/fig.sh \] && source ~/.fig/fig.sh"
INSTALLATION3="#### END FIG ENV VARIABLES ####"

/usr/bin/sed -i '' -e "s/$INSTALLATION1//g" ~/.profile ~/.zprofile ~/.bash_profile ~/.bashrc ~/.zshrc
# change delimeter to '#' in order to escape '/'
/usr/bin/sed -i '' -e "s#$INSTALLATION2##g" ~/.profile ~/.zprofile ~/.bash_profile ~/.bashrc ~/.zshrc
/usr/bin/sed -i '' -e "s/$INSTALLATION3//g" ~/.profile ~/.zprofile ~/.bash_profile ~/.bashrc ~/.zshrc

echo "Removing fish integration"
FISH_INSTALLATION='contains $HOME/.fig/bin $fish_user_paths; or set -Ua fish_user_paths $HOME/.fig/bin'

sed -i '' -e "s|$FISH_INSTALLATION||g" ~/.config/fish/config.fish
rm ~/.config/fish/conf.d/fig.fish

echo "Removing SSH integration"
SSH_CONFIG_PATH=~/.ssh/config
SSH_TMP_PATH=$SSH_CONFIG_PATH'.tmp'
# make backup?
cp $SSH_CONFIG_PATH $SSH_CONFIG_PATH'.backup'

# remove all three implementation
START="# Fig SSH Integration: Enabled"

END1="(fig bg:ssh ~/.ssh/%r@%h:%p &)"
END2="fig bg:ssh ~/.ssh/%r@%h:%p &"
END3="# End of Fig SSH Integration"

if grep -q "$END1" $SSH_CONFIG_PATH
  then
  cat $SSH_CONFIG_PATH | /usr/bin/sed -e '\|'"$START"'|,\|'"$END1"'|d' > $SSH_TMP_PATH
elif grep -q "$END2" $SSH_CONFIG_PATH
  then
  cat $SSH_CONFIG_PATH | /usr/bin/sed -e '\|'"$START"'|,\|'"$END2"'|d' > $SSH_TMP_PATH
elif grep -q "$END3" $SSH_CONFIG_PATH
  then
  cat $SSH_CONFIG_PATH | /usr/bin/sed -e '\|'"$START"'|,\|'"$END3"'|d' > $SSH_TMP_PATH
else
  echo "SSH Integration appears not to be installed. Ignoring."
fi

mv $SSH_TMP_PATH $SSH_CONFIG_PATH

echo "Removing TMUX integration"
TMUX_CONFIG_PATH=~/.tmux.conf
TMUX_TMP_PATH=$TMUX_CONFIG_PATH'.tmp'

TMUX_START="# Fig Tmux Integration: Enabled"
TMUX_END="# End of Fig Tmux Integration"

cat $TMUX_CONFIG_PATH | /usr/bin/sed -e '\|'"$TMUX_START"'|,\|'"$TMUX_END"'|d' > $TMUX_TMP_PATH

mv $TMUX_TMP_PATH $TMUX_CONFIG_PATH

echo "Remove Hyper plugin, if it exists"
HYPER_CONFIG=~/.hyper.js
test -f $HYPER_CONFIG && sed -i '' -e 's/"fig-hyper-integration",//g' $HYPER_CONFIG
test -f $HYPER_CONFIG && sed -i '' -e 's/"fig-hyper-integration"//g' $HYPER_CONFIG

#fig bg:event "Uninstall App"
echo "Finished removing fig resources. You may now delete the Fig app by moving it to the Trash."
#fig bg:alert "Done removing Fig resources." "You may now delete the Fig app by moving it to the Trash."

rm -rf /Applications/Fig.app
