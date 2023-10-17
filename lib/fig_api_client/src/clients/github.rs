use std::borrow::Cow;

use fig_request::{
    reqwest_client,
    Error,
};
use serde::{
    Deserialize,
    Serialize,
};
use url::Url;

/// A Github repo with the form `"owner/repo"`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHub {
    pub owner: Cow<'static, str>,
    pub repo: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub content_type: String,
    pub browser_download_url: Url,
}

impl GitHub {
    pub const fn new(owner: Cow<'static, str>, repo: Cow<'static, str>) -> Self {
        Self { owner, repo }
    }
}

impl Serialize for GitHub {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let GitHub { owner, repo } = self;
        serializer.serialize_str(&format!("{owner}/{repo}"))
    }
}

impl<'de> Deserialize<'de> for GitHub {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut parts = s.split('/');
        let owner = parts.next().ok_or_else(|| serde::de::Error::custom("missing owner"))?;
        let repo = parts.next().ok_or_else(|| serde::de::Error::custom("missing repo"))?;
        Ok(GitHub {
            owner: owner.to_owned().into(),
            repo: repo.to_owned().into(),
        })
    }
}

impl std::fmt::Display for GitHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.owner, self.repo)
    }
}

impl GitHub {
    pub fn readme_url(&self) -> Url {
        Url::parse(&format!(
            "https://raw.githubusercontent.com/{}/{}/HEAD/README.md",
            self.owner, self.repo
        ))
        .unwrap()
    }

    pub fn repository_url(&self) -> Url {
        Url::parse(&format!("https://github.com/{}/{}", self.owner, self.repo)).unwrap()
    }

    pub fn git_url(&self) -> Url {
        Url::parse(&format!("https://github.com/{}/{}.git", self.owner, self.repo)).unwrap()
    }

    pub async fn latest_release(&self) -> Result<GithubRelease, Error> {
        let url = Url::parse(&format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.owner, self.repo
        ))
        .unwrap();
        let Some(client) = reqwest_client::reqwest_client(true) else {
            return Err(Error::NoClient);
        };
        let release = client
            .get(url)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Accept", "application/vnd.github.raw+json")
            .send()
            .await?
            .json::<GithubRelease>()
            .await?;
        Ok(release)
    }
}
