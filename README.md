# Fig Monorepo

The Fig monorepo houses most of the core Fig code for the Fig desktop app
and CLI. Several projects live here:

- `proto/` - [protocol buffer](https://developers.google.com/protocol-buffers/) message specification for inter-process communication
- `figterm/` - figterm, our headless terminal/pseudoterminal that intercepts the user’s terminal edit buffer.
- `fig_cli/` - the fig CLI, allows users to interface with Fig from the command line
- `fig_desktop/` - the Rust desktop app (currently working on Linux, with work being done on MacOS/Windows), uses [`tao`](https://docs.rs/tao/latest/tao/)/[`wry`](https://docs.rs/wry/latest/wry/) for windowing/webviews
- `rust-lib/` - Rust libraries
    - `alacritty_terminal` - Our internal fork of the [alacritty internal terminal implementation](https://github.com/alacritty/alacritty/tree/master/alacritty_terminal), used for figterm ansi parsing and screen tracking
    - `fig_auth` - AWS credential management, mostly used for fetching the current auth token
    - `fig_color` - Used for figterm to parse colors in terminal output
    - `fig_directories` - A wrapper over [`dirs`](https://docs.rs/dirs/latest/dirs/) that provides standard locations for fig folders
    - `fig_integrations` - Fig's system integrations (ssh, dotfiles, etc)
    - `fig_ipc` - Defines the fig wire protocol and standard locations for sockets
    - `fig_log` - Defines standard ways to log errors using [`tracing`](https://docs.rs/tracing/latest/tracing/)
    - `fig_proto` - The protocol buffer definitions compiled to Rust
    - `fig_settings` - Utilities for interacting with figs remote/local settings and local state
    - `fig_telemetry` - Used to report telemetry to segment and [`sentry`](https://docs.rs/sentry/latest/sentry/)
    - `fig_util` - Misc other utilites that are useful in mutiple projects (Terminal, Shell enums, etc)
    - `system_socket` - A light wrapper over `UnixSockets` that allows them to be used in Windows projects as well
    - `viu` - An internal fork of [`viu`](https://github.com/atanunq/viu) to provide displaying of images in the terminal
- `typescript-api-bindings/` - The protocol buffer bindings for typescript
- `fig/` - Core logic for the legacy macOS desktop app

## Setup

### Prerequisites 

- MacOS
  - Xcode 13 or later
  - Brew

### 1. Clone repo

Using GitHub CLI:

```bash
gh repo clone withfig/macos
```

Using Git SSH:
```bash
git clone git@github.com:withfig/macos
```

### 2. Install platform dependencies

This is all the dep

For Debian/Ubuntu:

```bash
sudo apt update
sudo apt install build-essential pkg-config jq dpkg curl wget cmake clang libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libdbus-1-dev libwebkit2gtk-4.0-dev valac libibus-1.0-dev libglib2.0-dev sqlite3
```

For Arch:

```bash
sudo pacman -Syu
sudo pacman -S --needed webkit2gtk base-devel curl wget openssl appmenu-gtk-module gtk3 libappindicator-gtk3 librsvg libvips cmake jq pkgconf
```

For Fedora:

```bash
sudo dnf check-update
sudo dnf install webkit2gtk3-devel.x86_64 openssl-devel curl wget libappindicator-gtk3 librsvg2-devel jq
sudo dnf group install "C Development Tools and Libraries"
```

For MacOS:

```bash
xcode-select --install
brew install swiftlint yarn jq
```

### 2. Install protobuf compilers.

See [proto/README.md](https://github.com/withfig/macos/blob/develop/proto/README.md)

### 3. Install Rust toolchain using [Rustup](https://rustup.rs): 

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup toolchain default stable
```

For MacOS development make sure the right targests are installed:

```bash
rustup target add x86_64-apple-darwin
rustup target add arm_64-apple-darwin
```

### 4. Setup precommit hooks

```bash
# Run `yarn` in root directory to add pre-commit hooks
yarn
```

### 5. XCode (MacOS)
 - You MUST be [added](https://appstoreconnect.apple.com/access/users) to Fig's Apple Developer account. 
 - Setup Xcode signing credentials

## Building and Running Projects

### MacOS App

Before building you may need to:

1. Install Swift Packages (in XCode: File > Packages > Refresh Package Cache).

2. Make protos:
   ```bash
   cd proto && make
   ```

You can build from the XCode UI directly or from the terminal:
```bash
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


## Git Branching Conventions

- Feature branches
    - `name/feature-name`
    - e.g. `grant/bug-fix`
- `develop` branch 
  - should be buildable and pass all lints
- `staging` branch 
  - used to deploy to beta/staging
- `master` branch is auto pushed to prod
  - used to deploy to develop
