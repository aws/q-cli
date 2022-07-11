use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

fn map_env_dir(path: &OsStr) -> Option<PathBuf> {
    let path = Path::new(path);
    path.is_absolute().then(|| path.to_path_buf())
}

/// The $HOME directory
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// The $HOME/.fig directory
pub fn fig_dir() -> Option<PathBuf> {
    match std::env::var_os("FIG_DOT_DIR") {
        Some(dot_dir) => map_env_dir(&dot_dir),
        None => home_dir().map(|p| p.join(".fig")),
    }
}

/// The $DATA/fig directory
pub fn fig_data_dir() -> Option<PathBuf> {
    match std::env::var_os("FIG_DATA_DIR") {
        Some(data_dir) => map_env_dir(&data_dir),
        None => dirs::data_local_dir().map(|path| path.join("fig")),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        home_dir().unwrap();
        assert_eq!(fig_dir().unwrap().file_name().unwrap(), ".fig");
        assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "fig");

        std::env::set_var("FIG_DOT_DIR", "/abc");
        std::env::set_var("FIG_DATA_DIR", "/def");

        assert_eq!(fig_dir().unwrap().file_name().unwrap(), "abc");
        assert_eq!(fig_data_dir().unwrap().file_name().unwrap(), "def");

        std::env::set_var("FIG_DOT_DIR", "abc");
        std::env::set_var("FIG_DATA_DIR", "def");

        assert_eq!(fig_dir(), None);
        assert_eq!(fig_data_dir(), None);
    }
}
