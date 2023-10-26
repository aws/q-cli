#!/bin/bash

cargo build --target=x86_64-apple-darwin --target=aarch64-apple-darwin --locked --release --package fig_input_method
mkdir -p build/CodeWhispererInputMethod.app/Contents/{MacOS,Resources}
lipo -create -output build/CodeWhispererInputMethod.app/Contents/MacOS/fig_input_method target/{x86_64,aarch64}-apple-darwin/release/fig_input_method
cp fig_input_method/Info.plist build/CodeWhispererInputMethod.app/Contents/
cp -r fig_input_method/resources/* build/CodeWhispererInputMethod.app/Contents/Resources/
