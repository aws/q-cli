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

# TODO: reenable once mise fixes http issues
# mise install --verbose
# eval "$(mise activate bash --shims)"

# clean up old install
rm -rf "$CARGO_HOME"
rm -rf "$RUSTUP_HOME"

curl --retry 5 --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
source "$CARGO_HOME/env"
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

cargo install tauri-cli@1.5.2 --locked

# create python venv and install dmgbuild
python3.11 -m venv .venv
source .venv/bin/activate
pip3 install dmgbuild==1.6.1

python3.11 build-scripts/main.py build \
  --output-bucket "${output_bucket:-}" \
  --signing-bucket "${signing_bucket:-}" \
  --signing-queue "${signing_queue:-}" \
  --apple-id-secret "${apple_id_secret:-}" \
  --aws-account-id "${aws_account_id:-}" \
  --signing-role-name "${signing_role_name:-}" \
  --stage-name "${stage_name:-}" \
  2>&1
