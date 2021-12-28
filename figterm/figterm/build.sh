#!/bin/bash

cargo build --target=aarch64-apple-darwin --release
cargo build --target=x86_64-apple-darwin --release
lipo -create -output figterm target/aarch64-apple-darwin/release/figterm target/x86_64-apple-darwin/release/figterm
