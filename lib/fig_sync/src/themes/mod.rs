use fig_util::directories;
use tracing::{
    debug,
    error,
    info,
};

use crate::git::{
    self,
    GitError,
};

const THEMES_REPO: &str = "https://github.com/withfig/themes.git";

pub async fn clone_or_update() -> Result<(), GitError> {
    match git::clone_git_repo_with_reference(THEMES_REPO, directories::themes_repo_dir().unwrap(), None).await {
        Ok(_) => {
            info!("Cloned themes repo");
        },
        Err(err) => {
            debug!("Error cloning themes repo: {err}");
            match git::update_git_repo_with_reference(directories::themes_repo_dir().unwrap(), None).await {
                Ok(_) => info!("Updated themes repo"),
                Err(err) => {
                    error!("Error updating themes repo: {err}");
                },
            };
        },
    }

    Ok(())
}
