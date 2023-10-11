#!/bin/bash
set -ex

rustflags=(
  "-C force-frame-pointers=yes"
)

if [[ -n "${LINKER-}" ]]; then
  rustflags+=(
    "-C link-arg=-fuse-ld=${LINKER}"
  )
fi

PLATFORM=$(uname)

if [[ "${PLATFORM}" == "Linux" ]]; then
  rustflags+=("-C link-arg=-Wl,--compress-debug-sections=zlib")
fi

export CARGO_INCREMENTAL=0
export CARGO_PROFILE_RELEASE_LTO=thin
export RUSTFLAGS="${rustflags[*]}"
export CARGO_NET_GIT_FETCH_WITH_CLI=true

if [[ "${PLATFORM}" == "Darwin" ]]; then
  export MACOSX_DEPLOYMENT_TARGET=10.13
fi
