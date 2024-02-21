#!/bin/bash

# Lightweight bash script as the entry point for build.py as part of a CodeBuild project
# Intended to be remotely invoked via SSM

set -eux

echo "$@"

export CI=1

while [[ $# -gt 0 ]]; do
  case $1 in
    --output-bucket)
      shift
      output_bucket=$1
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
    --stage-name)
      shift
      stage_name="$1"
      ;;
  esac
  shift
done

export CARGO_HOME="$PWD/../.cargo"
RUSTUP_HOME="$PWD/../.rustup"

mise install
eval "$(mise activate bash --shims)"

# clean up old install
rm -rf "$CARGO_HOME"
rm -rf "$RUSTUP_HOME"

curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
source "$CARGO_HOME/env"
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

cargo install tauri-cli@1.5.2 --locked

pip install dmgbuild==1.6.1

build_params_json="$(
    jq -n \
        --arg output_bucket "${output_bucket:-}" \
        --arg signing_bucket "${signing_bucket:-}" \
        --arg signing_queue "${signing_queue:-}" \
        --arg apple_id_secret "${apple_id_secret:-}" \
        --arg aws_account_id "${aws_account_id:-}" \
        --arg signing_role_name "${signing_role_name:-}" \
        --arg stage_name "${stage_name:-}" \
        '{
            "output_bucket": (if $output_bucket == "" then null else $output_bucket end),
            "signing_bucket": (if $signing_bucket == "" then null else $signing_bucket end),
            "signing_queue": (if $signing_queue == "" then null else $signing_queue end),
            "apple_id_secret": (if $apple_id_secret == "" then null else $apple_id_secret end),
            "aws_account_id": (if $aws_account_id == "" then null else $aws_account_id end),
            "signing_role_name": (if $signing_role_name == "" then null else $signing_role_name end),
            "stage_name": (if $stage_name == "" then null else $stage_name end),
        }'
)"

python3 build-scripts/build.py "${build_params_json}" 2>&1
