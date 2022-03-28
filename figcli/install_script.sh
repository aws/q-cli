#!/usr/bin/env bash
set -e

## <script src="https://get-fig-io.s3.us-west-1.amazonaws.com/readability.js"></script>
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

# Fail fast with a concise message when not using bash or zsh
# Single brackets are needed here for POSIX compatibility
if [ -z "${BASH_VERSION:-}" ] && [ -z "${ZSH_VERSION:-}" ]; then
  abort "Bash is required to interpret this script."
fi

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

# Whether or not to install minimal remote install.
IS_REMOTE=1

# Ensure the user has the necessary tools to install fig
function check_for_command() {
    if ! command -v $1 >/dev/null; then
        abort "Please install $1 before running this script."
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
        abort "Neither curl nor wget found. Please install one of them."
    fi
}

# The directory where the binary will be installed
function global_install_directory() {
    case "$(uname -s)" in
        Linux*|Darwin*)
            _install_dir="/usr/local/bin"
            ;;
        *)
            abort "Unknown OS type: $_ostype"
            ;;
    esac

    # Check that the directory is in the PATH
    if ! echo "$PATH" | grep -q "$_install_dir"; then
        abort "Please add $_install_dir to your PATH."
    fi

    # Return the install directory
    echo "$_install_dir"
}

# The directory where the binary will be installed
function install_directory() {
    case "$(uname -s)" in
        Linux*|Darwin*)
            echo "${HOME}/.local/bin"
            ;;
        *)
            abort "Unknown OS type: $_ostype"
            ;;
    esac
}

# The directory where the binary is downloaded to
download_dir="$(mktemp -d)"

# Download the latest binary
download_file "${DOWNLOAD_URL}" "${download_dir}/fig"

# Check the files is a valid binary
if file "${download_dir}/fig" | grep -q "executable"; then
    # Make the binary executable
    chmod +x "${download_dir}/fig"
else
    abort "Your platform and architecture (${PLATFORM}-${ARCH}) is unsupported."
fi

INSTALL_DIR="$(install_directory)"
mkdir -p "${INSTALL_DIR}"
mv "${download_dir}/fig" "${INSTALL_DIR}"

if [[ -n "${SSH_TTY}" ]]; then
    printf "On remote machine, installing fig shell integrations only.\n"
    "${INSTALL_DIR}/fig" install --dotfiles
    if [ $? -ne 0 ]; then
        abort "Failed to install shell integrations."
    fi
else
    "${INSTALL_DIR}/fig" install
    if [ $? -ne 0 ]; then
        abort "Failed to install fig"
    fi

    printf "\n${MAGENTA}➜${RESET} ${BOLD}Next steps:${RESET}\n"
    printf "  Run ${MAGENTA}fig login${RESET} to login to your fig account\n"
    printf "  Run ${MAGENTA}fig${RESET} to start editing your dotfiles\n"
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
# Install scripts for Homebrew, Docker & Sentry were also used as reference.
