#!/usr/bin/env bash

#  ssh.sh
#  fig
#
#  Created by Matt Schrage on 1/12/21.
#  Copyright © 2021 Matt Schrage. All rights reserved.

#
SSH_CONFIG_PATH=~/.ssh/config
# If this comment is changed, it must also be updated in the Swift app
# when checking if integration is already setup
# and in the uninstall script.
START="# Fig SSH Integration: Enabled"
END="# End of Fig SSH Integration"

grep -q "$START" $SSH_CONFIG_PATH && echo "Fig is already integrated with SSH." && exit

# When this is updated, make sure ssh.html reflects these changes
CONFIG="$START
Include ~/.fig/ssh
$END"


mkdir -p ~/.ssh
touch $SSH_CONFIG_PATH
echo -e "$CONFIG\n\n$(cat $SSH_CONFIG_PATH)" > $SSH_CONFIG_PATH
echo Added Fig Integration to $SSH_CONFIG_PATH!
defaults write com.mschrage.fig SSHIntegrationEnabled -bool TRUE

fig bg:event "Setup SSH Integration"
