#!/bin/bash

set -eux

export VERSION
VERSION=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | select(.name == "fig_desktop") | .version')
echo "Version ${VERSION}"

prepare_bundle() {
    echo Checking for binaries
    ls "$FIGTERM" >/dev/null
    ls "$FIG_CLI" >/dev/null
    if [[ $IS_MINIMAL = 0 ]]; then
        ls "$FIG_DESKTOP" >/dev/null

        echo Installing icons
        for res in 16 22 24 32 48 64 128 256 512; do
            install -Dm644 "fig_desktop/icons/${res}x${res}.png" \
                "build/usr/share/icons/hicolor/${res}x${res}/apps/fig.png"
        done
        install -Dm644 fig_desktop/icons/512x512.png build/usr/share/pixmaps/fig.png
    fi

    echo Copying bundle files
    mkdir -p build/usr/bin
    cp "$FIG_CLI" build/usr/bin/fig
    cp "$FIGTERM" build/usr/bin/figterm
    cp -r bundle/linux/minimal/. build/
    if [[ $IS_MINIMAL = 0 ]]; then
        cp -r bundle/linux/desktop/. build/
        cp "$FIG_DESKTOP" build/usr/bin/fig_desktop
    fi
}

gen_manifest() {
    echo Generating install manifest
    mkdir -p build/usr/share/fig
    if [[ $IS_MINIMAL = 0 ]]; then
        VARIANT=full
    else
        VARIANT=minimal
    fi
    jq -n \
        --arg ib "$1" \
        --arg pa "$(date -Iseconds)" \
        --arg va "$VARIANT" \
        --arg ve "$VERSION" \
        --arg kd "$KIND" \
        --arg dc "$(cargo metadata --format-version 1 --no-deps | jq -r .metadata.channel)" \
        '{managed_by: $ib, packaged_at: $pa, packaged_by: "fig", variant: $va, version: $ve, kind: $kd, default_channel: $dc}' \
        >build/usr/share/fig/manifest.json
}
