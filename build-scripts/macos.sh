#!/bin/bash

set -e

. build-scripts/common.sh

KIND=dmg gen_manifest dmg
mv build/usr/share/fig/manifest.json fig_desktop/manifest.json
