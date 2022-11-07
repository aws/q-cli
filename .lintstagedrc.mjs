export default {
  "*.{rs,toml}": () => [
    "cargo +nightly fmt --check -- --color always",
    "cargo clippy --locked --color always -- -D warnings",
  ],
  "*.proto": "clang-format -n --Werror",
  "*": "typos",
};
