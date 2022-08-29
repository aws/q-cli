#!/bin/bash
set -ex

rustflags=(
  "-C force-frame-pointers=yes"
)

if [[ ! -z "$LINKER" ]]; then
  rustflags+=(
    "-C link-arg=-fuse-ld=${LINKER}"
  )
fi

PLATFORM=$(uname)

if [[ "$PLATFORM" == "Linux" ]]; then
  rustflags+=("-C link-arg=-Wl,--compress-debug-sections=zlib")
fi

cat <<EOF >>"${BASH_ENV}"
export CARGO_INCREMENTAL="0"
export CARGO_PROFILE_RELEASE_LTO="thin"
export RUSTFLAGS="${rustflags[*]}"
EOF
