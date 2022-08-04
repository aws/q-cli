use std::path::PathBuf;

use anyhow::Result;
use fig_util::directories;
use tracing::{
    error,
    info,
};

use crate::git;

const THEMES_REPO: &str = "https://github.com/withfig/themes.git";

fn themes_repo_directory() -> Result<PathBuf> {
    Ok(directories::fig_data_dir()?.join("themes"))
}

pub fn themes_directory() -> Result<PathBuf> {
    Ok(themes_repo_directory()?.join("themes"))
}

pub async fn clone_or_update() -> Result<()> {
    match git::clone_git_repo_with_reference(THEMES_REPO, themes_repo_directory().unwrap(), None).await {
        Ok(_) => {
            info!("Cloned themes repo");
        },
        Err(err) => {
            error!("Error cloning themes repo: {err}");
            match git::update_git_repo_with_reference(themes_repo_directory().unwrap(), None).await {
                Ok(_) => info!("Updated themes repo"),
                Err(err) => {
                    error!("Error updating themes repo: {err}");
                },
            };
        },
    }

    Ok(())
}
