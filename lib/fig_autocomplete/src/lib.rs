use std::fs;
use std::io::Cursor;
use std::path::Path;

use fig_api_client::autocomplete::{
    get_zipped_specs_from,
    AUTOCOMPLETE_REPO,
};
use thiserror::Error;
use zip::read::ZipArchive;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Zip Error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Request Error: {0}")]
    Request(#[from] fig_request::Error),
    #[error("Settings Error: {0}")]
    Settings(#[from] fig_settings::Error),
    #[error("Directory Error: {0}")]
    Directory(#[from] fig_util::directories::DirectoryError),
    #[error("Fig desktop is not installed so autocomplete specs will not be updated")]
    DesktopAppNotInstalled,
    #[error("Could not find the root directory of the zip file")]
    RootDirNotFound,
}

pub static SETTINGS_SPEC_VERSION: &str = "autocomplete.last-spec-version";

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for dir_entry in fs::read_dir(src)?.flatten() {
        if let Ok(file_type) = dir_entry.file_type() {
            if file_type.is_dir() {
                copy_dir_all(dir_entry.path(), dst.as_ref().join(dir_entry.file_name()))?;
            } else {
                fs::copy(dir_entry.path(), dst.as_ref().join(dir_entry.file_name()))?;
            }
        };
    }
    Ok(())
}

pub async fn _update_spec_store(force: bool) -> Result<(), Error> {
    if !fig_util::manifest::is_full() {
        return Err(Error::DesktopAppNotInstalled);
    }

    let temp_dir = std::env::temp_dir().join("codewhisperer").join("autocomplete_specs");
    tokio::fs::create_dir_all(&temp_dir).await?;
    let latest_release = AUTOCOMPLETE_REPO.latest_release().await?;

    if force
        || fig_settings::state::get::<String>(SETTINGS_SPEC_VERSION).ok().flatten()
            != Some(latest_release.tag_name.clone())
    {
        let data = get_zipped_specs_from(&latest_release).await?;
        ZipArchive::new(Cursor::new(data))?.extract(&temp_dir)?;
        // first file of temp dir is for sure the root file of the zip
        let zip_root_dir_name = fs::read_dir(&temp_dir)?
            .next()
            .ok_or(Error::RootDirNotFound)??
            .file_name();

        let spec_dir = fig_util::directories::autocomplete_specs_dir()?;
        tokio::fs::remove_dir_all(&spec_dir).await.ok();
        if let Some(parent) = spec_dir.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        copy_dir_all(temp_dir.join(&zip_root_dir_name), spec_dir)?;
        tokio::fs::remove_dir_all(&temp_dir).await?;

        fig_settings::state::set_value(SETTINGS_SPEC_VERSION, latest_release.tag_name)?;
    };
    Ok(())
}
