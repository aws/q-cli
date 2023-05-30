use std::fs;
use std::io::Cursor;

use fig_api_client::autocomplete::{
    get_zipped_specs_from,
    AUTOCOMPLETE_REPO,
};
use zip::read::ZipArchive;

pub static SETTINGS_SPEC_VERSION: &str = "autocomplete.last-spec-version";

pub async fn update_spec_store(force: bool) -> anyhow::Result<()> {
    let temp_dir = std::env::temp_dir().join("fig").join("autocomplete_specs");
    tokio::fs::create_dir_all(&temp_dir).await?;
    let latest_release = AUTOCOMPLETE_REPO.latest_release().await?;

    if force
        || fig_settings::settings::get::<String>(SETTINGS_SPEC_VERSION)? != Some(latest_release.tag_name.to_owned())
    {
        let data = get_zipped_specs_from(&latest_release).await?;
        ZipArchive::new(Cursor::new(data))?.extract(&temp_dir)?;
        // first file of temp dir is for sure the root file of the zip
        let zip_root_dir_name = fs::read_dir(&temp_dir)?.next().unwrap()?.file_name();

        fig_settings::settings::set_value(SETTINGS_SPEC_VERSION, latest_release.tag_name)?;
        tokio::fs::remove_dir_all(fig_util::directories::autocomplete_specs_dir()?).await?;
        tokio::fs::rename(
            temp_dir.join(zip_root_dir_name),
            fig_util::directories::autocomplete_specs_dir()?,
        )
        .await?;
    };
    Ok(())
}
