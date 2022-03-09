use std::path::PathBuf;

/// The $HOME directory
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// The $HOME/.fig directory
pub fn fig_dir() -> Option<PathBuf> {
    home_dir().map(|p| p.join(".fig"))
}

/// The $DATA/fig directory
pub fn fig_data_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|path| path.join("fig"))
}
