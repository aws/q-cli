//! Download and updating of plugins

use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use flume::Receiver;
use git2::build::RepoBuilder;
use git2::{
    FetchOptions,
    RemoteCallbacks,
    Repository,
};
use parking_lot::RwLock;
use reqwest::{
    IntoUrl,
    Url,
};
use serde::{
    Deserialize,
    Serialize,
};
use tokio::io::AsyncWriteExt;

use crate::plugins::manifest::GitReference;
use crate::util::checksum::GitChecksum;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DownloadMetadata {
    Git { git_repo: Url, checksum: GitChecksum },
    Remote { url: Url },
    Local { path: PathBuf },
}

#[must_use]
pub fn plugin_data_dir() -> Option<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            fig_directories::home_dir().map(|dir| dir.join(".local").join("share").join("fig").join("plugins"))
        } else {
            fig_directories::fig_data_dir().map(|dir| dir.join("plugins"))
        }
    }
}

pub async fn download_remote_file(
    url: impl IntoUrl,
    directory: impl AsRef<Path>,
    name: impl AsRef<Path>,
) -> Result<()> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;

    let file_path = directory.as_ref().join(name);

    let mut file = tokio::fs::File::create(&file_path).await?;
    file.write_all(body.as_bytes()).await?;

    Ok(())
}
struct GitProgress {
    total_objects: usize,
    received_objects: usize,
    total_deltas: usize,
    indexed_deltas: usize,
    received_bytes: usize,
}

struct GitUpdatedTips {
    refspecs: Vec<String>,
}

struct GitFetchStatus {
    git_progress: RwLock<GitProgress>,
    git_updated_tips: RwLock<GitUpdatedTips>,
}

fn git_fetch_options() -> (FetchOptions<'static>, Arc<GitFetchStatus>, Receiver<String>) {
    let (sideband_progress_tx, sideband_progress_rx) = flume::unbounded();

    let mut fetch_options = FetchOptions::new();
    let mut remote_callbacks = RemoteCallbacks::new();

    let git_fetch_status = Arc::new(GitFetchStatus {
        git_progress: RwLock::new(GitProgress {
            total_objects: 0,
            received_objects: 0,
            total_deltas: 0,
            indexed_deltas: 0,
            received_bytes: 0,
        }),
        git_updated_tips: RwLock::new(GitUpdatedTips { refspecs: vec![] }),
    });

    let git_fetch_status_clone = git_fetch_status.clone();
    remote_callbacks.transfer_progress(move |progress| {
        let mut git_fetch_status = git_fetch_status_clone.git_progress.write();

        git_fetch_status.total_objects = progress.total_objects();
        git_fetch_status.received_objects = progress.received_objects();
        git_fetch_status.total_deltas = progress.total_deltas();
        git_fetch_status.indexed_deltas = progress.indexed_deltas();
        git_fetch_status.received_bytes = progress.received_bytes();

        true
    });

    let git_fetch_status_clone = git_fetch_status.clone();
    remote_callbacks.update_tips(move |refspec, _, _| {
        let mut git_updated_tips = git_fetch_status_clone.git_updated_tips.write();

        git_updated_tips.refspecs.push(refspec.to_string());

        true
    });

    remote_callbacks.sideband_progress(move |bytes| {
        if let Ok(bytes) = std::str::from_utf8(bytes) {
            sideband_progress_tx
                .send_timeout(bytes.to_string(), Duration::from_millis(1))
                .ok();
        }

        true
    });

    fetch_options.remote_callbacks(remote_callbacks);

    (fetch_options, git_fetch_status, sideband_progress_rx)
}

pub fn update_git_repo(repository: &Repository) -> Result<()> {
    for remote_name in repository.remotes()?.iter().flatten() {
        let mut remote = repository.find_remote(remote_name)?;

        let refspecs = remote.fetch_refspecs()?;
        let refspecs_vec: Vec<_> = refspecs.iter().flatten().collect();

        let (mut fetch_options, _, _) = git_fetch_options();

        remote.fetch(&refspecs_vec, Some(&mut fetch_options), None)?;
    }

    Ok(())
}

pub async fn clone_git_repo(url: impl IntoUrl, directory: impl AsRef<Path>) -> Result<GitChecksum> {
    let temp_directory = tempfile::tempdir_in(plugin_data_dir().unwrap())?;

    let sha_id = {
        let (fetch_options, _, _) = git_fetch_options();

        let repo = tokio::task::block_in_place(|| {
            RepoBuilder::new()
                .fetch_options(fetch_options)
                .clone(url.as_str(), temp_directory.path())
        })?;

        let sha_id = repo.head()?.peel_to_commit()?.id().to_string();

        sha_id
    };

    tokio::fs::rename(temp_directory.path(), directory.as_ref()).await?;

    Ok(GitChecksum::new(sha_id))
}

pub fn set_reference(repository: &Repository, reference: &GitReference) -> Result<()> {
    let refname = match reference {
        GitReference::Branch(branch) => branch,
        GitReference::Tag(tag) => tag,
        GitReference::Commit(commit) => commit,
    };

    let (object, reference) = repository.revparse_ext(refname).expect("Object not found");

    repository.checkout_tree(&object, None).expect("Failed to checkout");

    match reference {
        // gref is an actual reference like branches or tags
        Some(gref) => repository.set_head(gref.name().unwrap()),
        // this is a commit, not a reference
        None => repository.set_head_detached(object.id()),
    }
    .expect("Failed to set HEAD");

    Ok(())
}

pub async fn clone_git_repo_with_reference(
    url: impl IntoUrl,
    directory: impl AsRef<Path>,
    reference: Option<&GitReference>,
) -> Result<()> {
    let directory = directory.as_ref();
    let url = url.into_url()?;

    if let Some(parent_directory) = directory.parent() {
        if !parent_directory.exists() {
            tokio::fs::create_dir_all(parent_directory).await?;
        }
    }

    if !directory.exists() {
        if let Err(err) = clone_git_repo(url, &directory).await {
            fig_telemetry::sentry::capture_anyhow(&err);
            return Err(err);
        }
    } else {
        anyhow::bail!("{} already exists", directory.display());
    }

    if let Some(reference) = reference {
        tokio::task::block_in_place(|| {
            set_reference(&Repository::open(directory)?, reference)?;
            anyhow::Ok(())
        })?;
    }

    Ok(())
}

pub async fn update_git_repo_with_reference(
    directory: impl AsRef<Path>,
    reference: Option<&GitReference>,
) -> Result<()> {
    let directory = directory.as_ref();
    if directory.exists() {
        tokio::task::block_in_place(|| {
            let repository = Repository::open(directory)?;
            update_git_repo(&repository)?;
            anyhow::Ok(())
        })?;
    }

    if let Some(reference) = reference {
        tokio::task::block_in_place(|| {
            set_reference(&Repository::open(directory)?, reference)?;
            anyhow::Ok(())
        })?;
    }

    Ok(())
}

pub async fn sideband_printer(sideband_rx: Receiver<String>) {
    tokio::spawn(async move {
        crossterm::execute!(std::io::stdout(), crossterm::cursor::Hide,).ok();
        while let Ok(line) = sideband_rx.recv_async().await {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
                crossterm::style::Print(line)
            )
            .ok();
            std::io::stdout().flush().ok();
        }
        crossterm::execute!(std::io::stdout(), crossterm::cursor::Show).ok();
    });
}

#[cfg(test)]
mod tests {
    use reqwest::Url;
    use tokio::process::Command;

    use super::*;
    use crate::plugins::manifest::GitHub;

    #[ignore]
    #[tokio::test]
    async fn test_download_remote_file_checksum_mismatch() {
        let url = "https://gist.githubusercontent.com/raw/916e80ae32717eeec18d2c7a50a13192";
        let directory = tempfile::tempdir().unwrap();

        let result = download_remote_file(url, directory.path(), "file").await;

        assert!(result.is_err());
    }

    #[ignore]
    #[tokio::test]
    async fn test_download_remote_file_checksum_valid() {
        let url = "https://gist.githubusercontent.com/raw/916e80ae32717eeec18d2c7a50a13192";
        let directory = tempfile::tempdir().unwrap();

        let result = download_remote_file(url, directory.path(), "file").await;

        assert!(result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_download_source_git() {
        let branch = "main";

        let directory = tempfile::tempdir().unwrap();

        clone_git_repo_with_reference(
            Url::parse("https://github.com/withfig/fig.git").unwrap(),
            directory.path().join("fig"),
            Some(&GitReference::Branch(branch.into())),
        )
        .await
        .unwrap();

        // Check that the branch is correct
        let branch_output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(directory.path().join("fig"))
            .output()
            .await
            .unwrap();

        let branch_stdout = String::from_utf8(branch_output.stdout).unwrap();

        assert_eq!(branch_stdout.trim(), "main");
    }

    #[ignore]
    #[tokio::test]
    async fn test_download_source_github() {
        let commit = "d112d75ecc1d867e7f223577c25c56f57f862c7b";
        let github = GitHub::new("withfig", "fig");

        let directory = tempfile::tempdir().unwrap();

        clone_git_repo_with_reference(
            github.git_url(),
            directory.path().join("fig"),
            Some(&GitReference::Commit(commit.into())),
        )
        .await
        .unwrap();

        // Check that the commit is correct
        let commit_output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(directory.path().join("fig"))
            .output()
            .await
            .unwrap();

        let commit_stdout = String::from_utf8(commit_output.stdout).unwrap();

        assert_eq!(commit_stdout.trim(), commit);
    }
}
