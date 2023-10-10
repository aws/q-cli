#!/bin/bash

echo "$@"

brew update
brew upgrade

rustup update

bash build-scripts/macos.sh
