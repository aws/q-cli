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

cargo install tauri-cli@1.5.2 --locked

build_params_json="$(
    jq -n \
        --arg allow_dev_functionality "${allow_dev_functionality:-}" \
        --arg output_bucket "${output_bucket:-}" \
        --arg tauri_private_key_secret "${tauri_private_key_secret:-}" \
        --arg tauri_private_key_password_secret "${tauri_private_key_password_secret:-}" \
        --arg signing_bucket "${signing_bucket:-}" \
        --arg signing_queue "${signing_queue:-}" \
        --arg apple_id_secret "${apple_id_secret:-}" \
        --arg aws_account_id "${aws_account_id:-}" \
        --arg signing_role_name "${signing_role_name:-}" \
        '{
            "allow_dev_functionality": $allow_dev_functionality,
            "output_bucket": $output_bucket,
            "tauri_private_key_secret": $tauri_private_key_secret,
            "tauri_private_key_password_secret": $tauri_private_key_password_secret,
            "signing_bucket": $signing_bucket,
            "signing_queue": $signing_queue,
            "apple_id_secret": $apple_id_secret,
            "aws_account_id": $aws_account_id,
            "signing_role_name": $signing_role_name
        }'
)"

python3 build-scripts/build.py "${build_params_json}"
