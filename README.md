

# Installation
### Prerequisites 
- Xcode 13 or later

1. Clone repo
```
git clone https://github.com/withfig/macos
```
2. Install protobuf compilers. See [proto/README.md](https://github.com/withfig/macos/blob/develop/proto/README.md)
```
cd proto && make`
```
3. Install Rust toolchain using [Rustup](https://rustup.rs): 
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```
rustup target add x86_64-apple-darwin
rustup target add arm_64-apple-darwin
```

6. Build project in Xcode
 - Setup Xcode signing credentials
    - You MUST be added to Fig's Apple Developer account. 
 - Install Swift Packages ( File > Packages > Refresh Package Cache)
 - Run Build (either from UI or terminal using the following command)
```
xcodebuild -scheme fig build
```
