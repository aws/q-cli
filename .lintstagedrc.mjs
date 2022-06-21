export default {
  "**/*.swift": ["swiftlint lint --fix --quiet 2>/dev/null", "swiftlint lint"],
  "**/*.rs": (files) => [
    `cargo +nightly fmt --check -- --color always ${files.join(" ")}`,
    "cargo clippy --color always -- -D warnings",
  ],
};
