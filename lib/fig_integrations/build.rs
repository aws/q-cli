const CODEX_FOLDER: &str = "src/shell/ghost_text";

// The order here is very specific, do no edit without understanding the implications
const CODEX_FILES: &[&str] = &[
    "guard_start.zsh",
    "LICENSE",
    "config.zsh",
    "util.zsh",
    "bind.zsh",
    "highlight.zsh",
    "widgets.zsh",
    "strategies/ghost_text.zsh",
    "strategies/completion.zsh",
    "strategies/history.zsh",
    "strategies/match_prev_cmd.zsh",
    "fetch.zsh",
    "async.zsh",
    "start.zsh",
    "guard_end.zsh",
];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = std::path::Path::new(&out_dir);

    let mut ghost_text = String::new();
    for file in CODEX_FILES {
        let path = std::path::Path::new(CODEX_FOLDER).join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        ghost_text.push_str(&std::fs::read_to_string(path).unwrap());
    }
    std::fs::write(out_dir.join("ghost_text.zsh"), ghost_text).unwrap();
}
