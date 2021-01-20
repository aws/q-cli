[ -d ~/Library/'Application Support'/iTerm2/ ] && mkdir -p ~/Library/'Application Support'/iTerm2/Scripts/AutoLaunch/
[ -d ~/Library/'Application Support'/iTerm2/Scripts/AutoLaunch ] && ln -s "$1/fig-iterm-integration.py" ~/Library/'Application Support'/iTerm2/Scripts/AutoLaunch/fig-iterm-integration.py
defaults write com.googlecode.iterm2 EnableAPIServer 1
