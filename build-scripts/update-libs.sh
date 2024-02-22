#!/usr/bin/env bash

# Update the internal brazil deps, since we dont use brazil we have to do this
# step manually and commit the changes to the repo.
#
# TODO: add telemetry updating, but we dont have a package to do this yet

set -e

TMP_FOLDER="$(mktemp -d)"

# AWSVectorConsolasRuntimeServiceRustClient version
CLIENT_VERSION="0.1.5"
PACKAGES=(
    amzn-codewhisperer-client
    amzn-codewhisperer-streaming-client
    amzn-consolas-client
)

for package in "${PACKAGES[@]}"; do
    URL="https://prod.artifactbrowser.brazil.aws.dev/api/v1/packages/AWSVectorConsolasRuntimeServiceRustClient/versions/${CLIENT_VERSION}.0/platforms/AL2_x86_64/flavors/DEV.STD.PTHREAD/rust1x/package/AWSVectorConsolasRuntimeServiceRustClient-${CLIENT_VERSION}.0/${package}-${CLIENT_VERSION}.crate?download=true"

    # download and extract the package
    mcurl -L "$URL" -o "${TMP_FOLDER}/${package}.tar.gz"
    tar -xzf "${TMP_FOLDER}/${package}.tar.gz" -C "${TMP_FOLDER}"

    # move the package to the right place
    rm -rf "lib/${package}"
    mv "${TMP_FOLDER}/${package}-${CLIENT_VERSION}" "lib/${package}"

    # clean up package
    rm -rf "lib/${package}/Cargo.toml."*
    sed '/resolver = "1"/d' "lib/${package}/Cargo.toml" > "lib/${package}/Cargo.toml.tmp"
    mv "lib/${package}/Cargo.toml.tmp" "lib/${package}/Cargo.toml"
done

cargo clippy --fix --allow-dirty
cargo +nightly fmt
