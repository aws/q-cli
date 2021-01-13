#!/usr/bin/env bash

#  ssh.sh
#  fig
#
#  Created by Matt Schrage on 1/12/21.
#  Copyright © 2021 Matt Schrage. All rights reserved.

#
SSH_CONFIG_PATH=~/.ssh/config
# If this comment is changed, it must also be updated in the Swift app
# when checking if integration is already setup.
COMMENT="# Fig SSH Integration: Enabled!"
grep -q "$COMMENT" $SSH_CONFIG_PATH && echo "Fig is already integrated with SSH." && exit

CONFIG="$COMMENT
Host *
    ControlPath ~/.ssh/tmp/%r@%h:%p
    ControlMaster auto
    ControlPersist 1
    PermitLocalCommand yes
    LocalCommand (fig bg:ssh ~/.ssh/tmp/%r@%h:%p &)"


mkdir -p ~/.ssh
touch SSH_CONFIG_PATH
echo -e "$CONFIG\n\n$(cat $SSH_CONFIG_PATH)" > $SSH_CONFIG_PATH
echo Added Fig Integration to $SSH_CONFIG_PATH!
defaults write com.mschrage.fig SSHIntegrationEnabled -bool TRUE

fig bg:event "Setup SSH Integration"
