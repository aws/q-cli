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
  esac
  shift
done

export CARGO_HOME=$PWD/../.cargo

curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
source $CARGO_HOME/env
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
rustup component add clippy

cargo install tauri-cli@1.5.2 --locked

bash build-scripts/macos.sh 2>&1

# If signing is requested, hande it
if [[ -n $signing_bucket && -n $signing_queue && -n $apple_id_secret ]]
then
    echo signing and notarizing...
    bash build-scripts/sign-and-rebundle-macos.sh "$signing_bucket" "$signing_queue" "$apple_id_secret" 2>&1
fi

if [[ -n $output_bucket ]]
then
    STAGING_LOCATION=s3://$output_bucket/staging/
    
    echo build complete, publishing to S3...
    aws s3 cp build/CodeWhisperer.dmg "${STAGING_LOCATION}"
fi
