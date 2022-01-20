#!/usr/bin/env bash
set -eu

## <script src="./readability.js"></script>
## <link href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/themes/prism-okaidia.min.css" rel="stylesheet" />
## <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/components/prism-core.min.js" data-manual></script>
## <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/components/prism-bash.min.js"></script>
## <style>body {color: #272822; background-color: #272822; font-size: 0.8em;} </style>

FIG_DOWNLOAD_Darwin_arm64="https://get.fig.io/bin/latest/darwin-arm64"
# FIG_DOWNLOAD_Darwin_universal="https://get.fig.io/bin/latest/darwin-universal"
FIG_DOWNLOAD_Darwin_x86_64="https://get.fig.io/bin/latest/darwin-x86_64"
# FIG_DOWNLOAD_Linux_aarch64="https://get.fig.io/bin/latest/linux-aarch64"
# FIG_DOWNLOAD_Linux_armv7="https://get.fig.io/bin/latest/linux-armv7"
# FIG_DOWNLOAD_Linux_i686="https://get.fig.io/bin/latest/linux-i686"
FIG_DOWNLOAD_Linux_x86_64="https://get.fig.io/bin/latest/linux-x86_64"
# FIG_DOWNLOAD_Windows_i686="https://get.fig.io/bin/latest/windows-i686.exe"
# FIG_DOWNLOAD_Windows_x86_64="https://get.fig.io/bin/latest/windows-x86_64.exe"

# Parse flags
while getopts ":hd" opt; do
    case $opt in
        h)
            echo "Usage: install_script.sh [-hd]"
            echo "  -h  Show this help text"
            echo "  -d  Enable debug mode"
            exit 0
            ;;
        d)
            echo "Debug mode"
            DEBUG=1
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

PLATFORM=`uname -s`
ARCH=`uname -m`

if [[ $PLATFORM == CYGWIN* ]] || [[ $PLATFORM == MINGW* ]] || [[ $PLATFORM == MSYS* ]]; then
  PLATFORM="Windows"
fi

# if [[ $PLATFORM == "Darwin" ]]; then
#     ARCH="universal"
# fi

if [[ $ARCH == armv8* ]] || [[ $ARCH == arm64* ]] || [[ $ARCH == aarch64* ]]; then
    ARCH="aarch64"
fi

if [[ $ARCH == armv6* ]] || [[ $ARCH == armv7* ]]; then
    ARCH="armv7"
fi

DOWNLOAD_URL_LOOKUP="SENTRY_DOWNLOAD_${PLATFORM}_${ARCH}"

# URL to download the latest version of the binary
DOWNLOAD_URL="${!DOWNLOAD_URL_LOOKUP:-}"

if [ x$DOWNLOAD_URL == x ]; then
  echo "error: your platform and architecture (${PLATFORM}-${ARCH}) is unsupported."
  exit 1
fi


# Download $1 to $2
function download_file() {
    if command -v curl &> /dev/null; then
        curl -s -L -o "$2" "$1"
        if [ $? -ne 0 ]; then
            echo "Failed to download $1"
            exit 1
        fi
    elif command -v wget &> /dev/null; then
        wget -q -O "$2" "$1"
        if [ $? -ne 0 ]; then
            echo "Failed to download $1"
            exit 1
        fi
    else
        echo "Neither curl nor wget found. Please install one of them."
        exit 1
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
        echo "Please add $_install_dir to your PATH."
        exit 1
    fi

    # Return the install directory
    echo "$_install_dir"
}

if [ -z "$DEBUG" ]; then
    # The directory where the binary is downloaded to
    INSTALL_DIR="$(mktemp -d)"

    # Download the latest binary
    download_file "${DOWNLOAD_URL}" "${download_dir}/dotfiles"

    # Make the binary executable and install it
    chmod +x "${download_dir}/dotfiles"
    sudo mv "${download_dir}/dotfiles" "$(install_directory)"
else
    make install
fi


if command -v dotfiles &> /dev/null; then
    sudo dotfiles install

    if [ $? -ne 0 ]; then
        echo "Failed to install dotfiles"
        exit 1
    fi

    echo "Successfully installed dotfiles"
    echo "Run 'dotfiles' to start using dotfiles"
else
    echo "Failed to install dotfiles. Command 'dotfiles' not found"
    exit 1
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
