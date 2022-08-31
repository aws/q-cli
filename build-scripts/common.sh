#!/bin/bash

export VERSION=$(sed -nr 's/^version[[:space:]]*=[[:space:]]*\"([^"]*)\"/\1/p' fig_desktop/Cargo.toml | head -1)
echo "Version ${VERSION}"

prepare_bundle() {
    echo Checking for binaries
    ls $FIGTERM >/dev/null
    ls $FIG_CLI >/dev/null
    if [[ $IS_HEADLESS = 0 ]]; then
        ls $FIG_DESKTOP >/dev/null

        echo Installing icons
        for res in 16 22 24 32 48 64 128 256 512; do
            install -Dm644 "fig_desktop/icons/${res}x${res}.png" \
                "build/usr/share/icons/hicolor/${res}x${res}/apps/fig.png"
        done
        install -Dm644 fig_desktop/icons/512x512.png build/usr/share/pixmaps/fig.png
    fi

    echo Copying bundle files
    mkdir -p build/usr/bin
    cp $FIG_CLI build/usr/bin/fig
    cp $FIGTERM build/usr/bin/figterm
    ln -s /usr/bin/figterm build/usr/bin/bash\ \(figterm\)
    ln -s /usr/bin/figterm build/usr/bin/fish\ \(figterm\)
    ln -s /usr/bin/figterm build/usr/bin/zsh\ \(figterm\)
    cp -r bundle/linux/headless/. build/
    if [[ $IS_HEADLESS = 0 ]]; then
        cp -r bundle/linux/desktop/. build/
        cp $FIG_DESKTOP build/usr/bin/fig_desktop
    fi
}

gen_manifest() {
    echo Generating install manifest
    mkdir -p build/usr/share/fig
    if [[ $IS_HEADLESS = 0 ]]; then
        VARIANT=desktop
    else
        VARIANT=headless
    fi
    jq -n \
        --arg ib "$1" \
        --arg pa $(date -Iseconds) \
        --arg va "$VARIANT" \
        '{managed_by: $ib, packaged_at: $pa, packaged_by: "fig", variant: $va}' \
        >build/usr/share/fig/manifest.json
}
