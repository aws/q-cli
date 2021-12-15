use std::path::PathBuf;

/// Get the path to `~/.fig`
pub fn fig_path() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(".fig");
    dir
}

pub fn get_term_bundle() -> Option<String> {
    match std::env::var("TERM_PROGRAM").ok().as_deref() {
        Some("iTerm.app") => Some("com.googlecode.iterm2".into()),
        Some("Apple_Terminal") => Some("com.apple.Terminal".into()),
        Some("Hyper") => Some("co.zeit.hyper".into()),
        Some("vscode") => match std::env::var("TERM_PROGRAM_VERSION").ok().as_deref() {
            Some(v) if v.contains("insiders") => Some("com.microsoft.vscode-insiders".into()),
            _ => Some("com.microsoft.vscode".into()),
        },
        _ => std::env::var("TERM_BUNDLE_IDENTIFIER").ok(),
    }
}
