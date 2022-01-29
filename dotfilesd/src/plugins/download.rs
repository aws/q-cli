//! Download and updating of plugins

use std::path::{Path, PathBuf};

use anyhow::Result;
use git2::Repository;
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::util::{
    checksum::{GitChecksum, Sha256Checksum},
    project_dir,
};

use super::manifest::{GitReference, GithubValue, ShellSource};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DownloadMetadata {
    Git {
        git_repo: url::Url,
        checksum: GitChecksum,
    },
    Remote {
        url: url::Url,
        checksum: Sha256Checksum,
    },
    Local {
        path: PathBuf,
    },
}

pub fn plugin_data_dir() -> Option<PathBuf> {
    let user_dirs = project_dir()?;
    let source_folder = user_dirs.data_local_dir().join("plugin_data");
    Some(source_folder)
}

async fn download_remote_file(
    url: impl IntoUrl,
    directory: impl AsRef<Path>,
    checksum: &Option<Sha256Checksum>,
) -> Result<Sha256Checksum> {
    let response = reqwest::get(url).await?;
    let body = response.text().await?;

    let computed_checksum = Sha256Checksum::compute(&body);

    if let Some(checksum) = checksum {
        if &computed_checksum != checksum {
            return Err(anyhow::anyhow!(
                "Checksum mismatch: {:?} != {:?}",
                computed_checksum,
                checksum
            ));
        }
    }

    let file_path = directory.as_ref().join(&computed_checksum.as_str());

    let mut file = tokio::fs::File::create(&file_path).await?;
    file.write_all(body.as_bytes()).await?;

    Ok(computed_checksum)
}

async fn clone_git_repo(
    url: impl IntoUrl,
    directory: impl AsRef<Path>,
    reference: Option<&GitReference>,
) -> Result<GitChecksum> {
    let repo = Repository::clone(url.as_str(), &directory)?;

    if let Some(reference) = reference {
        let refname = match reference {
            GitReference::Branch(branch) => branch.to_string(),
            GitReference::Tag(tag) => tag.to_string(),
            GitReference::Commit(commit) => commit.to_string(),
        };

        let (object, reference) = repo.revparse_ext(&refname).expect("Object not found");

        repo.checkout_tree(&object, None)
            .expect("Failed to checkout");

        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => repo.set_head(gref.name().unwrap()),
            // this is a commit, not a reference
            None => repo.set_head_detached(object.id()),
        }
        .expect("Failed to set HEAD");
    }

    let sha_id = repo.head()?.peel_to_commit()?.id().to_string();

    Ok(GitChecksum::new(sha_id))
}

impl ShellSource {
    pub async fn download_source(&self, directory: impl AsRef<Path>) -> Result<DownloadMetadata> {
        tokio::fs::create_dir_all(&directory).await?;

        match self {
            ShellSource::Git { git, reference } => {
                let checksum = clone_git_repo(git.as_str(), directory, reference.as_ref()).await?;
                Ok(DownloadMetadata::Git {
                    git_repo: git.clone(),
                    checksum,
                })
            }
            ShellSource::Github { github, reference } => match github {
                GithubValue::GithubRepo(github_repo) => {
                    let github_url = github_repo.git_url();
                    let checksum =
                        clone_git_repo(github_url.as_str(), directory, reference.as_ref()).await?;
                    Ok(DownloadMetadata::Git {
                        git_repo: github_url,
                        checksum,
                    })
                }
                _ => {
                    return Err(anyhow::anyhow!("Non-normalized GitHub source"));
                }
            },
            ShellSource::Local { path: _ } => {
                // TODO: Determine what to do here
                todo!()
            }
            ShellSource::Gist { gist, checksum } => {
                let checksum = download_remote_file(gist.raw_url(), directory, checksum).await?;
                Ok(DownloadMetadata::Remote {
                    url: gist.raw_url(),
                    checksum,
                })
            }
            ShellSource::Remote { remote, checksum } => {
                let checksum = download_remote_file(remote.as_ref(), directory, checksum).await?;
                Ok(DownloadMetadata::Remote {
                    url: remote.clone(),
                    checksum,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::{io::AsyncReadExt, process::Command};

    use crate::plugins::manifest::{Gist, GitHub};

    use super::*;

    #[tokio::test]
    async fn test_download_remote_file() {
        let url = "https://gist.githubusercontent.com/raw/916e80ae32717eeec18d2c7a50a13192";
        let directory = tempfile::tempdir().unwrap();

        let checksum = download_remote_file(url, directory.path(), &None)
            .await
            .unwrap();

        assert_eq!(
            checksum.as_str(),
            "5b892a87c0cc8279a0469dfde36b5b80de1de4c9e9a9d8211a93aae789b26391"
        );

        // Read the file
        let file_path = directory.path().join(checksum.as_str());
        let mut file = tokio::fs::File::open(&file_path).await.unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).await.unwrap();

        assert!(contents.contains("echo \"hello from figrc\""));
    }

    #[tokio::test]
    async fn test_download_remote_file_checksum_mismatch() {
        let url = "https://gist.githubusercontent.com/raw/916e80ae32717eeec18d2c7a50a13192";
        let directory = tempfile::tempdir().unwrap();
        let checksum = Sha256Checksum::new("invalid_checksum");

        let result = download_remote_file(url, directory.path(), &Some(checksum)).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_remote_file_checksum_valid() {
        let url = "https://gist.githubusercontent.com/raw/916e80ae32717eeec18d2c7a50a13192";
        let directory = tempfile::tempdir().unwrap();
        let checksum =
            Sha256Checksum::new("5b892a87c0cc8279a0469dfde36b5b80de1de4c9e9a9d8211a93aae789b26391");

        let result = download_remote_file(url, directory.path(), &Some(checksum)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_download_source_git() {
        let branch = "main";

        let source = ShellSource::Git {
            git: "https://github.com/withfig/fig".try_into().unwrap(),
            reference: Some(GitReference::Branch(branch.into())),
        };

        let directory = tempfile::tempdir().unwrap();

        source.download_source(directory.path()).await.unwrap();

        // Check that the branch is correct
        let branch_output = Command::new("git")
            .args(&["branch", "--show-current"])
            .current_dir(directory.path())
            .output()
            .await
            .unwrap();

        let branch_stdout = String::from_utf8(branch_output.stdout).unwrap();

        assert_eq!(branch_stdout.trim(), "main");
    }

    #[tokio::test]
    async fn test_download_source_github() {
        let commit = "d112d75ecc1d867e7f223577c25c56f57f862c7b";

        let source = ShellSource::Github {
            github: GithubValue::GithubRepo(GitHub::new("withfig", "fig")),
            reference: Some(GitReference::Commit(commit.into())),
        };

        let directory = tempfile::tempdir().unwrap();

        source.download_source(directory.path()).await.unwrap();

        // Check that the commit is correct
        let commit_output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(directory.path())
            .output()
            .await
            .unwrap();

        let commit_stdout = String::from_utf8(commit_output.stdout).unwrap();

        assert_eq!(commit_stdout.trim(), commit);
    }

    #[tokio::test]
    async fn test_download_source_local() {
        let _source = ShellSource::Local {
            path: "./".try_into().unwrap(),
        };

        let _directory = tempfile::tempdir().unwrap();

        // source.download_source(directory.path()).await.unwrap();
    }

    #[tokio::test]
    async fn test_download_source_gist() {
        let source = ShellSource::Gist {
            gist: Gist::new("916e80ae32717eeec18d2c7a50a13192"),
            checksum: None,
        };

        let directory = tempfile::tempdir().unwrap();

        source.download_source(directory.path()).await.unwrap();
    }

    #[tokio::test]
    async fn test_download_source_remote() {
        let source = ShellSource::Remote {
            remote: "https://gist.githubusercontent.com/raw/916e80ae32717eeec18d2c7a50a13192"
                .try_into()
                .unwrap(),
            checksum: None,
        };

        let directory = tempfile::tempdir().unwrap();

        source.download_source(directory.path()).await.unwrap();
    }
}
