export default {
  "*.{rs,toml}": () => [
    "cargo +nightly fmt --check -- --color always",
    "cargo clippy --locked --color always -- -D warnings",
  ],
  "*.proto": () => [
    "cd proto && buf lint && buf format --error-code > /dev/null",
  ],
  "*": "typos",
};
