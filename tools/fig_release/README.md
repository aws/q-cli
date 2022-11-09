# Fig Release

### Bump version

Bumps the version of the Cargo.toml and Cargo.lock files.

```bash
cargo run -p fig_release -- bump
```

### Publish

Publishes the current branch to the index using the current channel

```bash
cargo run -p fig_release -- publish macos
```

## TODO

## Build Branch

Builds the current branch and uploads the artifacts to S3

```bash
cargo run -p fig_release -- build-branch macos
```
