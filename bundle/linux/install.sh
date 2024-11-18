#!/bin/sh

# Installs the q and qterm into place on the user's machine
# and installs the recommended integrations

set -o errexit
set -o nounset

SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"

log_error() {
    printf '\e[31m[ERROR]\e[0m %s\n' "$1" >&2
}

target_triple() {
    BUILD_INFO_PATH="$SCRIPT_DIR/BUILD-INFO"
    if [ ! -f "$BUILD_INFO_PATH" ]; then
        log_error "BUILD-INFO file not found."
        return 1
    fi

    target_triple_line=$(grep '^BUILD_TARGET_TRIPLE=' "$BUILD_INFO_PATH")
    if [ -z "$target_triple_line" ]; then
        log_error "BUILD_TARGET_TRIPLE not found in BUILD-INFO."
        return 1
    else
        echo "${target_triple_line#BUILD_TARGET_TRIPLE=}"
    fi
}

target_triple_uname() {
    target_triple=$(target_triple)
    case "$target_triple" in
        *linux*)
            echo "Linux"
            ;;
        *darwin*)
            echo "Darwin"
            ;;
        *windows*)
            echo "Windows"
            ;;
        *)
            log_error "Could not determine OS."
            return 1
            ;;
    esac
}

is_target_triple_gnu() {
    target_triple=$(target_triple)
    if [ "${target_triple##*-}" = "gnu" ]; then
        return 0
    else
        return 1
    fi
}

# checks that the system has atleast glibc 2.34
check_glibc_version() {
    if [ -f /lib64/libc.so.6 ]; then
        LIBC_PATH=/lib64/libc.so.6
    elif [ -f /lib/libc.so.6 ]; then
        LIBC_PATH=/lib/libc.so.6
    elif [ -f /usr/lib/x86_64-linux-gnu/libc.so.6 ]; then
        LIBC_PATH=/usr/lib/x86_64-linux-gnu/libc.so.6
    else
        log_error "Could not find glibc."
        return 1
    fi

    glibc_version=$("$LIBC_PATH" | sed -n 's/^GNU C Library (.*) stable release version \([0-9]*\)\.\([0-9]*\).*$/\1.\2/p')

    if [ -z "$glibc_version" ]; then
        log_error "Could not determine glibc version."
        return 1
    else
        IFS='.' read -r major minor << EOF
$glibc_version
EOF
        if [ -z "$minor" ]; then
            minor=0
        fi
        if [ "$major" -gt 2 ] || { [ "$major" -eq 2 ] && [ "$minor" -ge 34 ]; }; then
            return 0
        else
            return 1
        fi
    fi
}

# checks that uname matches the target triple
if [ "$(uname)" != "$(target_triple_uname)" ]; then
    log_error "This archive is built for a $(target_triple_uname) system."
    exit 1
fi

if is_target_triple_gnu && ! check_glibc_version; then
    log_error "This release built for a GNU system with glibc 2.34 or newer, try installing the musl version of the CLI."
    exit 1
fi

if [ -n "${Q_INSTALL_GLOBAL:-}" ]; then
    install -m 755 "$SCRIPT_DIR/bin/q" /usr/local/bin/
    install -m 755 "$SCRIPT_DIR/bin/qterm" /usr/local/bin/

    /usr/local/bin/q integrations install dotfiles
    /usr/local/bin/q setup --global
else
    mkdir -p "$HOME/.local/bin"

    install -m 755 "$SCRIPT_DIR/bin/q" "$HOME/.local/bin/"
    install -m 755 "$SCRIPT_DIR/bin/qterm" "$HOME/.local/bin/"

    "$HOME/.local/bin/q" setup
fi
