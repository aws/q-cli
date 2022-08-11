#!/usr/bin/env bash

## <script src="https://get-fig-io.s3.us-west-1.amazonaws.com/readability.js"></script>
## <link href="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/themes/prism-okaidia.min.css" rel="stylesheet" />
## <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/components/prism-core.min.js" data-manual></script>
## <script src="https://cdnjs.cloudflare.com/ajax/libs/prism/1.16.0/components/prism-bash.min.js"></script>
## <style>body {color: #272822; background-color: #272822; font-size: 0.8em;} </style>

if [ -z "${BASH_VERSION:-}" ] && [ -z "${ZSH_VERSION:-}" ]; then
  echo "Bash is required to interpret this script."
else
  function unsupported() {
    echo $1
    echo "If you would like support for this, please create a GitHub issue here: https://github.com/withfig/fig/issues/new/choose"
  }

  function helpmsg() {
    echo $1
    echo "If you'd like help, email us at hello@fig.io."
  }
  
  function install_fig() {
    FIG_DOWNLOAD_DIR="https://get-fig-io.s3.us-west-1.amazonaws.com/bin/latest"
    
    ARCH=`uname -m`
    PLATFORM=`uname -s`
    
    if [[ $ARCH == armv8* ]] || [[ $ARCH == arm64* ]] || [[ $ARCH == aarch64* ]]; then
      ARCH="aarch64"
    elif [[ $ARCH == x86_64* ]] || [[ $ARCH == amd64* ]]; then
      ARCH="x86_64"
    else
      unsupported "Unsupported architecture $ARCH."
      return 1
    fi
    
    if [[ $PLATFORM == Darwin* ]]; then
      PLATFORM="apple-darwin"
      INSTALL_DIR="${HOME}/.local/bin"
    elif [[ $PLATFORM == Linux* ]]; then
      PLATFORM="unknown-linux-gnu"
      INSTALL_DIR="${HOME}/.local/bin"

      LDD_VERSION=$(ldd --version | head -1 | rev | cut -d' ' -f1 | rev)
      LDD_MAJOR=$(echo $LDD_VERSION | cut -f1 -d'.')
      LDD_MINOR=$(echo $LDD_VERSION | cut -f2 -d'.')

      if [[ $ARCH == "aarch64" ]]; then
        if (( $LDD_MAJOR < 2 )) || ( (( $LDD_MAJOR == 2 )) && (( $LDD_MINOR < 31 )) ); then
          unsupported "Outdated glibc version $LDD_VERSION. On $ARCH machines fig requires at least glibc 2.31"
          return 1
        fi
      elif [[ $ARCH == "x86_64" ]]; then
        if (( $LDD_MAJOR < 2 )) || ( (( $LDD_MAJOR == 2 )) && (( $LDD_MINOR < 23 )) ); then
          unsupported "Outdated glibc version $LDD_VERSION. On $ARCH machines fig requires at least glibc 2.23"
          return 1
        fi
      fi
    elif [[ $PLATFORM == CYGWIN* ]] || [[ $PLATFORM == MINGW* ]] || [[ $PLATFORM == MSYS* ]]; then
      PLATFORM="pc-windows-msvc"
      unsupported "Fig currently does not support windows."
      return 1
    else
      unsupported "Unsupported platform $PLATFORM."
      return 1
    fi

    # URL to download the latest version of the binary
    DOWNLOAD_URL="$FIG_DOWNLOAD_DIR/$ARCH-$PLATFORM"
    DOWNLOAD_DIR="$(mktemp -d)"
    
    if command -v curl &> /dev/null; then
      curl -Lso "${DOWNLOAD_DIR}/fig" "${DOWNLOAD_URL}"
    elif command -v wget &> /dev/null; then
      wget -qO "${DOWNLOAD_DIR}/fig" "${DOWNLOAD_URL}"
    else
      echo "Could not find curl or wget to download fig from"
      echo "  ${DOWNLOAD_URL} "
      echo "Please install one and try again."
      return 1
    fi
    
    if [[ ! -f "${DOWNLOAD_DIR}/fig" ]]; then
      helmsg "Failed to download binary for ${PLATFORM}-${ARCH}."
      return 1
    fi
  
    mkdir -p "${INSTALL_DIR}"
    mv "${DOWNLOAD_DIR}/fig" "${INSTALL_DIR}"

    if ! chmod +x "${INSTALL_DIR}/fig"; then
      helpmsg "Failed to make fig binary executable"
      return 1
    fi
    
    if [[ $- == *i* ]]; then
      "${INSTALL_DIR}/fig" install --dotfiles --force
    else 
      "${INSTALL_DIR}/fig" install --dotfiles --force --no-confirm
    fi

    if [[ $? -ne 0 ]]; then
      helpmsg "Failed to install shell integrations."
      return 1
    fi

    # Source integrations in current shell.
    if [[ -n "${BASH_VERSION}" ]]; then
      [[ -f "$HOME/.fig/shell/bashrc.post.bash" ]] && . "$HOME/.fig/shell/bashrc.post.bash"
    elif [[ -n "${ZSH_VERSION}" ]]; then
      [[ -f "$HOME/.fig/shell/zshrc.post.zsh" ]] && . "$HOME/.fig/shell/zshrc.post.zsh"
    fi
  }

  install_fig
fi

# ------------------------------------------
#   Notes
# ------------------------------------------
#
# This script contains hidden JavaScript which is used to improve
# readability in the browser (via syntax highlighting, etc), right-click
# and "View source" of this page to see the entire bash script!
