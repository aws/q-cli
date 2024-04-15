#!/bin/sh

# Installs the cw and cwterm into place on the user's machine
# and installs the recommended integtations

set -o errexit
set -o nounset

# If not on linux error
if [ "$(uname)" != "Linux" ]; then
    echo "This script only works on Linux"
    exit 1
fi

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"

mkdir -p "$HOME/.local/bin"

install -m 755 "$SCRIPT_DIR/bin/cw" "$HOME/.local/bin/"
install -m 755 "$SCRIPT_DIR/bin/cwterm" "$HOME/.local/bin/"

"$HOME/.local/bin/cw" install
