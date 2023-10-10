#!/bin/bash

set -eux

echo "$@"

whoami

# brew update
# brew upgrade

export CARGO_HOME=$PWD/../.cargo

curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
source $CARGO_HOME/env
rustup target add aarch64-apple-darwin
rustup component add clippy

bash build-scripts/macos.sh
