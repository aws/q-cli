# Fig Monorepo

The Fig monorepo houses most of the core Fig code for the Fig desktop app
and CLI. Several projects live here:

- `fig/` - Core logic for the macOS desktop app
- `proto/` - [protocol
    buffer](https://developers.google.com/protocol-buffers/) message specification for inter-process communication
- `figterm/` - figterm, our headless terminal/pseudoterminal that
    intercepts the user’s terminal edit buffer.
- `fig_cli/` - the fig CLI, allows users to interface with Fig from the
    command line
- `rust-lib/` - Rust utilities, used in figterm and the fig CLI

## Setup
### Prerequisites 
- Xcode 13 or later
- Brew
- Yarn + Node

### 1. Clone repo
```
git clone git@github.com:withfig/macos
```
### 2. Install protobuf compilers.

See [proto/README.md](https://github.com/withfig/macos/blob/develop/proto/README.md)

### 3. Install Rust toolchain using [Rustup](https://rustup.rs): 

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure all supported macOS targets are installed:
rustup target add x86_64-apple-darwin
rustup target add arm_64-apple-darwin
```

### 4. Setup precommit hooks

```
# Install swiftlint
brew install swiftlint

# Run `yarn` in root directory to add pre-commit hooks
yarn
```

### 5. XCode
 - You MUST be [added](https://appstoreconnect.apple.com/access/users) to Fig's Apple Developer account. 
 - Setup Xcode signing credentials

## Building and Running Projects

### MacOS App

Before building you may need to:
1. Install Swift Packages (in XCode: File > Packages > Refresh Package
Cache).
2. Make protos:
   ```
   cd proto && make
   ```

You can build from the XCode UI directly or from the terminal:
```
xcodebuild -scheme fig build
```

### figterm

Run
```
make install
```
This will build the project and copy it to the correct place.

### fig CLI

Run
```
make install-native
```
This will build the project and copy it to the correct place.

## Publish 

When publishing a new version (pushing to master) of the app, be sure to bump `figcli` if required bacause it is needed to automatically generate specs.