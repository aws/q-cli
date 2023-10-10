#!/bin/bash

set -eux

echo "$@"

whoami

# brew update
# brew upgrade

rustup update

bash build-scripts/macos.sh
