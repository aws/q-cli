use anyhow::Result;
use directories::ProjectDirs;
use globset::{Glob, GlobSet, GlobSetBuilder};

pub mod checksum;
pub mod shell;
pub mod terminal;

pub fn project_dir() -> Option<ProjectDirs> {
    directories::ProjectDirs::from("io", "Fig", "Fig Cli")
}

pub fn glob(patterns: &[impl AsRef<str>]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern.as_ref())?);
    }
    Ok(builder.build()?)
}
