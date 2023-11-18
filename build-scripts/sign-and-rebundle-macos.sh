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

BUILD_DIR="./build"
app=$(ls -d1 "$BUILD_DIR/CodeWhisperer.app")
dmg=$(ls -1 "$BUILD_DIR/CodeWhisperer.dmg")

if [ -z "$app" ] || [ -z "$dmg" ]; then
  echo "Build artifact(s) not present, bailing on signing"
  exit 1
fi

echo "Working on $app and $dmg ..."

# Sign the application
sign_file "$app" app

# Notarize the application
notarize_file "$app"

# Rebundle the dmg file with the signed and notarized application
rebundle_dmg "$dmg" "$app"

# Sign the dmg
sign_file "$dmg" dmg

# Notarize the dmg
notarize_file "$dmg"

echo "All good!!"

