#!/bin/bash

set -eux

BUCKET_NAME="$1"                # e.g. fig-io-desktop-ec-signing-230592382359
SIGNING_REQUEST_QUEUE_NAME="$2" # e.g. fig-io-desktop-signing-requests
NOTARIZING_SECRET_ID="$3"       # e.g. fig-io-desktop-notarizing-apple-id
AWS_ACCOUNT_ID="$4"             # e.g. 230592382359 
SIGNING_ROLE_NAME="$5"          # e.g. codewhisperer-ec-signing-role

. build-scripts/signing-functions.sh "$BUCKET_NAME" "$SIGNING_REQUEST_QUEUE_NAME" "$NOTARIZING_SECRET_ID" "$AWS_ACCOUNT_ID" "$SIGNING_ROLE_NAME"

if ! get_secrets; then
    echo "Problem obtaining secrets"
    exit 1
fi

cargo build --target=x86_64-apple-darwin --target=aarch64-apple-darwin --locked --release --package fig_input_method
mkdir -p build/CodeWhispererInputMethod.app/Contents/{MacOS,Resources}
lipo -create -output build/CodeWhispererInputMethod.app/Contents/MacOS/fig_input_method target/{x86_64,aarch64}-apple-darwin/release/fig_input_method
cp fig_input_method/Info.plist build/CodeWhispererInputMethod.app/Contents/
cp -r fig_input_method/resources/* build/CodeWhispererInputMethod.app/Contents/Resources/

BUILD_DIR="./build"
app=$(ls -d1 "$BUILD_DIR/CodeWhispererInputMethod.app")

if [ -z "$app" ]; then
  echo "Build artifact(s) not present, bailing on signing"
  exit 1
fi

echo "Working signing $app ..."

# Sign the application
sign_file "$app" ime

# Notarize the application
notarize_file "$app"
