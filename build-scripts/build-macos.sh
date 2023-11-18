#!/bin/bash

# Lightweight bash script to build NSCC on MacOS as part of a CodeBuild project
# Intended to be remotely invoked via SSM

set -eux

echo "$@"

export CI=1

while [[ $# -gt 0 ]]; do
  case $1 in
    --allow-dev-functionality)
      allow_dev_functionality=true
      ;;
    --output-bucket)
      shift
      output_bucket=$1
      ;;
    --tauri-private-key-secret)
      shift
      tauri_private_key_secret=$1
      ;;
    --tauri-private-key-password-secret)
      shift
      tauri_private_key_password_secret=$1
      ;;
    --signing-bucket)
      shift
      signing_bucket=$1
      ;;
     --signing-queue)
      shift
      signing_queue=$1
      ;;   
    --apple-id-secret)
      shift
      apple_id_secret=$1
      ;;
    --aws-account-id)
      shift
      aws_account_id="$1"
      ;;
    --signing-role-name)
      shift
      signing_role_name="$1"
      ;;
  esac
  shift
done

export CARGO_HOME="$PWD/../.cargo"
RUSTUP_HOME="$PWD/../.rustup"

# clean up old install
rm -rf "$CARGO_HOME"
rm -rf "$RUSTUP_HOME"

curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
source "$CARGO_HOME/env"
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup component add clippy

cargo install tauri-cli@1.5.2 --locked
cargo install cargo-license@0.4.2 --locked

bash build-scripts/macos.sh "$signing_bucket" "$signing_queue" "$apple_id_secret" "$aws_account_id" "$signing_role_name" 2>&1

# If signing is requested, handle it
if [[ -n "$signing_bucket" && -n "$signing_queue" && -n "$apple_id_secret" ]]; then
    echo signing and notarizing...
    bash build-scripts/sign-and-rebundle-macos.sh "$signing_bucket" "$signing_queue" "$apple_id_secret" "$aws_account_id" "$signing_role_name" 2>&1
fi

shasum -a 256 build/CodeWhisperer.dmg | awk '{printf $1}' > build/CodeWhisperer.dmg.sha256

if [[ -n $output_bucket ]]; then
    STAGING_LOCATION="s3://$output_bucket/staging/"
    
    echo build complete, publishing to S3...
    aws s3 cp build/CodeWhisperer.dmg "${STAGING_LOCATION}"
    aws s3 cp build/CodeWhisperer.dmg.sha256 "${STAGING_LOCATION}"
fi
