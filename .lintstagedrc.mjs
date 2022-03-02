export default {
  "**/*.swift": ["swiftlint lint --fix --quiet 2>/dev/null", "swiftlint lint"],
  "**/*.rs": (files) => [
    `cargo fmt -- ${files.join(" ")}`,
    "cargo clippy -- -D warnings",
  ],
};
