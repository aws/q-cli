//! Misc utilities used throughout

use std::path::PathBuf;

/// Get the path to `~/.fig`
pub fn fig_path() -> Option<PathBuf> {
    dirs::home_dir().map(|mut dir| {
        dir.push(".fig");
        dir
    })
}

/// Gets the term_bundle
///
/// Only usable on MacOs
#[cfg(target_os = "macos")]
pub fn get_term_bundle() -> Option<std::borrow::Cow<'static, str>> {
    match std::env::var("TERM_PROGRAM").ok().as_deref() {
        Some("iTerm.app") => Some("com.googlecode.iterm2".into()),
        Some("Apple_Terminal") => Some("com.apple.Terminal".into()),
        Some("Hyper") => Some("co.zeit.hyper".into()),
        Some("vscode") => match std::env::var("TERM_PROGRAM_VERSION").ok().as_deref() {
            Some(v) if v.contains("insiders") => Some("com.microsoft.vscode-insiders".into()),
            _ => Some("com.microsoft.vscode".into()),
        },
        Some("Tabby") => Some("org.tabby".into()),
        _ => std::env::var("__CFBundleIdentifier").ok().map(|s| s.into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::fig_path;

    #[test]
    fn fig_path_test() {
        assert!(fig_path().unwrap().ends_with(".fig"));
    }

    #[test]
    #[cfg(all(target = "macos", feature = "desktop-tests"))]
    fn term_bundle_test() {
        use super::get_term_bundle;

        get_term_bundle().unwrap();
    }
}
