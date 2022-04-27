/// Gets the `term_bundle`
///
/// Only usable on `MacOs`
#[cfg(target_os = "macos")]
#[must_use]
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
        _ => std::env::var("__CFBundleIdentifier").ok().map(Into::into),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(all(target = "macos", feature = "desktop-tests"))]
    fn term_bundle_test() {
        use super::get_term_bundle;

        get_term_bundle().unwrap();
    }
}
