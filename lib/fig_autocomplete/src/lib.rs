use std::fs;
use std::io::Cursor;

use anyhow::Context;
use fig_api_client::autocomplete::{
    get_zipped_specs_from,
    AUTOCOMPLETE_REPO,
};
use zip::read::ZipArchive;

pub static SETTINGS_SPEC_VERSION: &str = "autocomplete.last-spec-version";

pub async fn update_spec_store(force: bool) -> anyhow::Result<()> {
    if !fig_util::manifest::is_full() {
        anyhow::bail!("Fig desktop is not installed so autocomplete specs will not be updated");
    }

    let temp_dir = std::env::temp_dir().join("fig").join("autocomplete_specs");
    tokio::fs::create_dir_all(&temp_dir).await?;
    let latest_release = AUTOCOMPLETE_REPO.latest_release().await?;

    if force
        || fig_settings::state::get::<String>(SETTINGS_SPEC_VERSION).ok().flatten()
            != Some(latest_release.tag_name.to_owned())
    {
        let data = get_zipped_specs_from(&latest_release).await?;
        ZipArchive::new(Cursor::new(data))?.extract(&temp_dir)?;
        // first file of temp dir is for sure the root file of the zip
        let zip_root_dir_name = fs::read_dir(&temp_dir)?
            .next()
            .context("Could not find the root directory of the zip file")??
            .file_name();

        let spec_dir = fig_util::directories::autocomplete_specs_dir()?;
        tokio::fs::remove_dir_all(&spec_dir).await.ok();
        if let Some(parent) = spec_dir.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::rename(temp_dir.join(zip_root_dir_name), spec_dir)
            .await
            .unwrap();

        fig_settings::state::set_value(SETTINGS_SPEC_VERSION, latest_release.tag_name)?;
    };
    Ok(())
}
