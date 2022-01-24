#!/usr/bin/env bash
set -eu

## <script src="./readability.js"></script>
## <link href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/themes/prism-okaidia.min.css" rel="stylesheet" />
## <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/components/prism-core.min.js" data-manual></script>
## <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/components/prism-bash.min.js"></script>
## <style>body {color: #272822; background-color: #272822; font-size: 0.8em;} </style>

MAGENTA="\033[1;35m"
BOLD="\033[1;1m"
RESET="\033[0m"

function abort() {
    echo $1
    echo "If you need help, please email us at ${BOLD}hello@fig.io${RESET}."
    exit 1
}

FIG_DOWNLOAD_DIR="https://get-fig-io.s3.us-west-1.amazonaws.com/bin/latest"

ARCH=`uname -m`
PLATFORM=`uname -s`

if [[ $ARCH == armv8* ]] || [[ $ARCH == arm64* ]] || [[ $ARCH == aarch64* ]]; then
    ARCH="aarch64"
fi

if [[ $ARCH == x86_64* ]] || [[ $ARCH == amd64* ]]; then
    ARCH="x86_64"
fi

if [[ $PLATFORM == Darwin* ]]; then
    PLATFORM="apple-darwin"
fi

if [[ $PLATFORM == Linux* ]]; then
    PLATFORM="unknown-linux-gnu"
fi

if [[ $PLATFORM == CYGWIN* ]] || [[ $PLATFORM == MINGW* ]] || [[ $PLATFORM == MSYS* ]]; then
    PLATFORM="pc-windows-msvc"
fi

# URL to download the latest version of the binary
DOWNLOAD_URL="$FIG_DOWNLOAD_DIR/$ARCH-$PLATFORM"

# Ensure the user has the nessisary tools to install dotfiles
function check_for_commands() {
    if ! command -v sudo >/dev/null; then
        abort "Please install sudo before running this script."
    fi
}

# Download url $1 to file $2
function download_file() {
    if command -v curl &> /dev/null; then
        curl -s -L -o "$2" "$1"
        if [ $? -ne 0 ]; then
            abort "Failed to download $1"
        fi
    elif command -v wget &> /dev/null; then
        wget -q -O "$2" "$1"
        if [ $? -ne 0 ]; then
            abort "Failed to download $1"
        fi
    else
        abort "Failed to download, neither curl nor wget is available"
    fi
}

# The directory where the binary will be installed
function install_directory() {
    _ostype="$(uname -s)"

    case "$_ostype" in
        Linux*)
            _ostype="linux"
            ;;
        Darwin*)
            _ostype="darwin"
            ;;
        *)
            echo "Unknown OS type: $_ostype"
            exit 1
            ;;
    esac

    case "$_ostype" in
        linux)
            _install_dir="/usr/local/bin"
            ;;
        darwin)
            _install_dir="/usr/local/bin"
            ;;
    esac

    # Check that the directory is in the PATH
    if ! echo "$PATH" | grep -q "$_install_dir"; then
        abort "Please add $_install_dir to your PATH."
    fi

    # Return the install directory
    echo "$_install_dir"
}

# The directory where the binary is downloaded to
download_dir="$(mktemp -d)"

# Download the latest binary
download_file "${DOWNLOAD_URL}" "${download_dir}/dotfiles"

# Check the files is a valid binary
if file "${download_dir}/dotfiles" | grep -q "executable"; then
    # Make the binary executable
    chmod +x "${download_dir}/dotfiles"
else
    abort "Your platform and architecture (${PLATFORM}-${ARCH}) is unsupported."
fi

check_for_commands

INSTALL_DIR="$(install_directory)"

printf "Installing dotfiles to ${BOLD}${INSTALL_DIR}${RESET}\n"
sudo mv "${download_dir}/dotfiles" "${INSTALL_DIR}"
printf "\n"

if command -v dotfiles &> /dev/null; then
    sudo dotfiles install

    if [ $? -ne 0 ]; then
        abort "Failed to install dotfiles"
    fi

    printf "\nRun ${MAGENTA}dotfiles${RESET} to get started!\n"
else
    abort "Failed to install dotfiles. Command 'dotfiles' not found."
fi

# ------------------------------------------
#   Notes
# ------------------------------------------
#
# This script contains hidden JavaScript which is used to improve
# readability in the browser (via syntax highlighting, etc), right-click
# and "View source" of this page to see the entire bash script!
#
# You'll also notice that we use the ":" character in the Introduction
# which allows our copy/paste commands to be syntax highlighted, but not
# ran. In bash : is equal to `true` and true can take infinite arguments
# while still returning true. This turns these commands into no-ops so
# when ran as a script, they're totally ignored.
#
# Credit goes to firebase.tools for the inspiration & much of the implementation.
