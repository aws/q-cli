#!/bin/bash

set -ex

cargo build -p fig_cli
cargo build -p figterm
cargo build -p fig_desktop

export FIG_CLI=target/debug/fig_cli
export FIGTERM=target/debug/figterm
export FIG_DESKTOP=target/debug/fig_desktop
export VERSION=0.0.0
export ARCH=x86_64
export IS_HEADLESS=0

set +ex
