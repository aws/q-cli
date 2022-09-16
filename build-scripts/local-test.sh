#!/bin/bash

set -ex

cargo build -rp fig_cli
cargo build -rp figterm
cargo build -rp fig_desktop

export FIG_CLI=target/release/fig_cli
export FIGTERM=target/release/figterm
export FIG_DESKTOP=target/release/fig_desktop
export VERSION=0.0.0
export ARCH=x86_64
export IS_HEADLESS=0

set +ex
