use std::path::{
    Path,
    PathBuf,
};

use fig_util::directories;
use time::OffsetDateTime;

pub fn get_default_backup_dir() -> Option<PathBuf> {
    let now = OffsetDateTime::now_utc()
        .format(time::macros::format_description!(
            "[year]-[month]-[day]_[hour]-[minute]-[second]"
        ))
        .ok()?;
    directories::home_dir()
        .map(|path| path.join(".fig.dotfiles.bak").join(now))
        .ok()
}

pub fn backup_file(path: impl AsRef<Path>, backup_dir: Option<impl Into<PathBuf>>) -> std::io::Result<()> {
    let pathref = path.as_ref();
    if pathref.exists() {
        let name: String = pathref.file_name().unwrap().to_string_lossy().into_owned();
        let dir = backup_dir
            .map(|dir| dir.into())
            .or_else(get_default_backup_dir)
            .unwrap();
        std::fs::create_dir_all(&dir)?;
        std::fs::copy(path, dir.join(name).as_path())?;
    }

    Ok(())
}
