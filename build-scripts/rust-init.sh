#!/bin/bash
set -eux

export LINKER="${LINKER:-lld}"
command -v "${LINKER}"

rustflags=(
  "-C link-arg=-fuse-ld=${LINKER}"
  "-C link-arg=-Wl,--compress-debug-sections=zlib"
  "-C force-frame-pointers=yes"
)

cat << EOF >> "${BASH_ENV}"
export CARGO_INCREMENTAL="0"
export CARGO_PROFILE_RELEASE_LTO="thin"
export RUSTFLAGS="${rustflags[*]}"
EOF

