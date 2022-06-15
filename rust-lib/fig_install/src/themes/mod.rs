use std::path::PathBuf;

use anyhow::Result;
use fig_directories::fig_data_dir;
use tracing::{
    error,
    info,
};

use crate::download;

const THEMES_REPO: &str = "https://github.com/withfig/themes.git";

fn themes_repo_directory() -> Option<PathBuf> {
    fig_data_dir().map(|dir| dir.join("themes"))
}

pub fn themes_directory() -> Option<PathBuf> {
    themes_repo_directory().map(|dir| dir.join("themes"))
}

pub async fn clone_or_update() -> Result<()> {
    match download::clone_git_repo_with_reference(THEMES_REPO, themes_repo_directory().unwrap(), None).await {
        Ok(_) => {
            info!("Cloned themes repo");
        },
        Err(err) => {
            error!("Error cloning themes repo: {err}");
            match download::update_git_repo_with_reference(themes_repo_directory().unwrap(), None).await {
                Ok(_) => info!("Updated themes repo"),
                Err(err) => {
                    error!("Error updating themes repo: {err}");
                },
            };
        },
    }

    Ok(())
}
