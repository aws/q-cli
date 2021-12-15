use std::path::PathBuf;

/// Get the path to `~/.fig`
pub fn fig_path() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(".fig");
    dir
}