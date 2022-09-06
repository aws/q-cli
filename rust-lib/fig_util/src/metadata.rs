use cfg_if::cfg_if;

/// Is Fig installed via headless install
pub fn is_headless() -> bool {
    // TODO(mia): Add metadata (replace current)
    cfg_if! {
        if #[cfg(target_os = "macos")] {
            !std::path::Path::new("/Applications/Fig.app/").exists()
        } else if #[cfg(target_os = "linux")] {
            which::which("fig_desktop").is_err()
        } else if #[cfg(target_os = "windows")] {
            false
        }
    }
}
