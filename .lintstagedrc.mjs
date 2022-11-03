export default {
  "*.swift": ["swiftlint lint --fix --quiet 2>/dev/null", "swiftlint lint"],
  "*.{rs,toml}": () => [
    "cargo +nightly fmt --check -- --color always",
    "cargo clippy --locked --color always -- -D warnings",
  ],
  "*.proto": "clang-format -n --Werror",
  "*": "typos"
};
