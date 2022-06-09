use std::path::{
    Path,
    PathBuf,
};

use anyhow::{
    Context,
    Result,
};
use time::OffsetDateTime;

pub fn get_default_backup_dir() -> Result<PathBuf> {
    let now = OffsetDateTime::now_utc().format(time::macros::format_description!(
        "[year]-[month]-[day]_[hour]-[minute]-[second]"
    ))?;
    fig_directories::home_dir()
        .map(|path| path.join(".fig.dotfiles.bak").join(now))
        .context("Could not get home dir")
}

pub fn backup_file(path: impl AsRef<Path>, backup_dir: Option<impl Into<PathBuf>>) -> Result<()> {
    let pathref = path.as_ref();
    if pathref.exists() {
        let name: String = pathref
            .file_name()
            .context(format!("Could not get filename for {}", pathref.display()))?
            .to_string_lossy()
            .into_owned();
        let dir = backup_dir
            .map(|dir| dir.into())
            .or_else(|| get_default_backup_dir().ok())
            .context("Could not get backup directory")?;
        std::fs::create_dir_all(&dir).context("Could not back up file")?;
        std::fs::copy(path, dir.join(name).as_path()).context("Could not back up file")?;
    }

    Ok(())
}
