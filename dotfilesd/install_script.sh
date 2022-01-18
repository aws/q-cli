#!/bin/sh

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

# URL to download the latest version of the binary
LATEST_BINARY='https://gist.githubusercontent.com/grant0417/916e80ae32717eeec18d2c7a50a13192/raw/9e0e44b994a30447d448b80063efb04f7be87d3c/gistfile1.txt'

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
    download_dir="$(mktemp -d)"

    # Download the latest binary
    download_file "${LATEST_BINARY}" "${download_dir}/dotfiles"

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
    echo "Failed to install dotfiles."
    exit 1
fi
