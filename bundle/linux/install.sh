#!/bin/sh

# Installs the q and qterm into place on the user's machine
# and installs the recommended integrations

set -o errexit
set -o nounset

# If not on linux error
if [ "$(uname)" != "Linux" ]; then
    echo "This script only works on Linux"
    exit 1
fi

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"

mkdir -p "$HOME/.local/bin"

install -m 755 "$SCRIPT_DIR/bin/q" "$HOME/.local/bin/"
install -m 755 "$SCRIPT_DIR/bin/qterm" "$HOME/.local/bin/"

"$HOME/.local/bin/q" install
