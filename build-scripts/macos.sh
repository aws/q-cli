#!/bin/bash

set -eux

# check that the user is in the git root dir
if [ ! -f "Config" ]; then
    echo "Please run this script from the root of the git repo"
    exit 1
fi

if [ -f ".env" ]; then
  . .env
fi

# security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_NAME" || echo "already exists"

# certificate_path="/tmp/certificate.p12"
# echo "$SIGNING_CERTIFICATE_P12_DATA" | base64 -d > $certificate_path
# security default-keychain -d user -s "$KEYCHAIN_NAME"

# security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_NAME"
# security import "$certificate_path" -f pkcs12 -k "$KEYCHAIN_NAME" -P "$SIGNING_CERTIFICATE_PASSWORD" -T /usr/bin/codesign -x 
# rm "$certificate_path"
# security set-key-partition-list -S apple-tool:,apple: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_NAME"

# identity=$(security find-identity -v -p codesigning | grep -o "Developer ID Application.*(${NOTARIZE_TEAM_ID})")
# export CODESIGNING_IDENTITY="$identity"

# security set-keychain-settings -lut 1200

export BUILD_DIR=build
export IS_HEADLESS=0
export TARGET=universal-apple-darwin

mkdir -p "$BUILD_DIR"

# build dashboard
pnpm build
rm -rf "$BUILD_DIR/dashboard"
cp -r apps/dashboard/dist "$BUILD_DIR/dashboard"

# build fig_cli
cargo build --target=x86_64-apple-darwin --target=aarch64-apple-darwin --locked --release --package fig_cli
lipo -create -output "$BUILD_DIR/cw-$TARGET" target/{x86_64,aarch64}-apple-darwin/release/fig_cli

# build figterm
cargo build --target=x86_64-apple-darwin --target=aarch64-apple-darwin --locked --release --package figterm
lipo -create -output "$BUILD_DIR/cwterm-$TARGET" target/{x86_64,aarch64}-apple-darwin/release/figterm
 
./build-scripts/ime.sh

# shellcheck source=common.sh
. build-scripts/common.sh
KIND=dmg gen_manifest dmg
mv build/usr/share/fig/manifest.json fig_desktop/manifest.json

config=$(jq -n \
  --arg cw "$(pwd)/$BUILD_DIR/cw" \
  --arg cwterm "$(pwd)/$BUILD_DIR/cwterm" \
  '{"tauri": {"bundle": {"externalBin": [$cw, $cwterm], "resources": ["manifest.json"]}}}'
)
cd fig_desktop
echo "${config}" > build-config.json
for entry in "authors" "homepage" "version"; do
  new="$(grep "^$entry = .*\$" ../Cargo.toml)"
  sed -i '' "s#$entry\.workspace = true#$new#g" Cargo.toml
done

BUILD_DIR="../$BUILD_DIR" cargo-tauri build --config ./build-config.json --target "$TARGET"

# clean up
rm build-config.json manifest.json
cd -

BUNDLE_DIR="$(pwd)/target/universal-apple-darwin/release/bundle/macos"
rm -rf "${BUNDLE_DIR}/CodeWhisperer.app"
mv "${BUNDLE_DIR}/codewhisperer_desktop.app" "${BUNDLE_DIR}/CodeWhisperer.app"

# Change the display name of the app
defaults write "${BUNDLE_DIR}/CodeWhisperer.app/Contents/Info.plist" CFBundleDisplayName CodeWhisperer
defaults write "${BUNDLE_DIR}/CodeWhisperer.app/Contents/Info.plist" CFBundleName CodeWhisperer

# Add codewhisperer:// association to bundle
plutil -insert CFBundleURLTypes -xml \
'<array>
  <dict>
    <key>CFBundleURLName</key>
    <string>com.amazon.codewhisperer</string>
    <key>CFBundleURLSchemes</key>
    <array>
      <string>codewhisperer</string>
    </array>
  </dict>
</array>' \
"${BUNDLE_DIR}/CodeWhisperer.app/Contents/Info.plist"

mkdir -p "${BUNDLE_DIR}/CodeWhisperer.app/Contents/Helpers/"
cp -r "${BUILD_DIR}/FigInputMethod.app" "${BUNDLE_DIR}/CodeWhisperer.app/Contents/Helpers/"

cp -r "${BUILD_DIR}/dashboard" "${BUNDLE_DIR}/CodeWhisperer.app/Contents/Resources/"

BUNDLE_PATH="${BUNDLE_DIR}/CodeWhisperer.app"

# codesign -v --timestamp --force --strict --options=runtime -s "$CODESIGNING_IDENTITY" -i io.fig.cli "$BUNDLE_PATH/Contents/MacOS/cw"
# codesign -v --timestamp --force --strict --options=runtime -s "$CODESIGNING_IDENTITY" -i io.fig.figterm "$BUNDLE_PATH/Contents/MacOS/cwterm" 
# codesign -v --timestamp --force --strict --options=runtime -s "$CODESIGNING_IDENTITY" -i io.fig.figterm "$BUNDLE_PATH/Contents/Helpers/FigInputMethod.app" 
# codesign -v --timestamp --force --strict --options=runtime -s "$CODESIGNING_IDENTITY" "$BUNDLE_PATH"
# codesign --verify --verbose --strict --entitlements entitlements.plist "$BUNDLE_PATH"

# ditto -c -k --keepParent "$BUNDLE_PATH" Cw.zip
# xcrun notarytool submit Cw.zip --apple-id "$NOTARIZE_USERNAME" --password "$NOTARIZE_PASSWORD" --team-id "$NOTARIZE_TEAM_ID" --wait
# rm -f Cw.zip
# xcrun stapler staple "$BUNDLE_PATH"
# Verify notarization ticket
# spctl -a -v "$BUNDLE_PATH"

  # --arg identity "$CODESIGNING_IDENTITY" \

FILE_CONTENTS=$(jq -n \
  --arg bundle "$BUNDLE_PATH" \
  '{
    "title": "CodeWhisperer",
    "icon": "VolumeIcon.icns",
    "background": "background.png",
    "icon-size": 160,
    "format": "ULFO",
    "window": {
      "size": {
        "width": 660,
        "height": 400
      }
    },
    "contents": [
      {
        "x": 180,
        "y": 170,
        "type": "file",
        "path": $bundle
      },
      {
        "x": 480,
        "y": 170,
        "type": "link",
        "path": "/Applications"
      }
    ]
  }'
)

SPEC_FILE="bundle/dmg/spec.json"
DMG="$BUILD_DIR/cw.dmg"

echo "$FILE_CONTENTS" > "$SPEC_FILE"
rm -f "$DMG"
pnpm appdmg "$SPEC_FILE" "$DMG"
rm "$SPEC_FILE"

# xcrun notarytool submit "$DMG" --apple-id "$NOTARIZE_USERNAME" --password "$NOTARIZE_PASSWORD" --team-id "$NOTARIZE_TEAM_ID" --wait
# spctl -a -t open --context context:primary-signature -v "$DMG"
