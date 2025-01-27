#!/usr/bin/env bash

set -eux

DIRECTORY="$1"
if [ -z "$DIRECTORY" ]; then
    echo "Usage: $0 <directory>"
    exit 1
fi

rsync -av --progress --delete \
    --exclude=node_modules \
    packages/ "$DIRECTORY/packages/"
rsync -av --progress --delete \
    --exclude=node_modules \
    proto/ "$DIRECTORY/packages/proto"

rm -rf "$DIRECTORY/packages/autocomplete-app" "$DIRECTORY/packages/dashboard"

fd . "$DIRECTORY/packages" --type file --exec sd 'workspace:\^' '*' 
fd . "$DIRECTORY/packages" --type file --exec sd 'workspace:*' '*'
fd . "$DIRECTORY/packages" --type file --exec sd 'workspace:\~' '*'

fd . "$DIRECTORY/packages" --type file --exec sd '@amzn\/' '@aws/amazon-q-developer-cli-'
fd . "$DIRECTORY/packages" --type file --exec sd '@aws\/amazon-q-developer-cli-' '@amzn/amazon-q-developer-cli-'


fd . "$DIRECTORY/packages/autocomplete" --type file --exec sd '@amzn\/amazon-q-developer-cli-autocomplete"' '@amzn/amazon-q-developer-cli-autocomplete-cloudshell"'
fd . "$DIRECTORY/packages/autocomplete" --type file --exec sd '  "private": true,\n' ''
