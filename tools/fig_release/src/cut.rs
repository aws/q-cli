use fig_settings::state;
use once_cell::sync::Lazy;
use semver::{
    BuildMetadata,
    Prerelease,
    Version,
};

use crate::utils::{
    gen_nightly,
    read_release_file,
    run,
    write_release_file,
    Channel,
};

pub static SOURCE_BRANCH: Lazy<String> = Lazy::new(|| {
    state::get_string("developer.release.sourceBranch")
        .ok()
        .flatten()
        .unwrap_or_else(|| "develop".into())
});
pub static BRANCH_PREFIX: Lazy<String> = Lazy::new(|| {
    state::get_string("developer.release.branchPrefix")
        .ok()
        .flatten()
        .unwrap_or_else(|| "".into())
});

pub fn nightly() -> eyre::Result<()> {
    run(&["git", "checkout", &SOURCE_BRANCH])?;
    run(&["git", "pull"])?;
    let mut release = read_release_file()?;
    let mut version = Version::parse(&release.version)?;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::new(&gen_nightly())?;
    release.version = version.to_string();
    release.channel = Some(Channel::Nightly);
    write_release_file(&release)?;
    run(&[
        "git",
        "checkout",
        "-b",
        &format!("{}{}", *BRANCH_PREFIX, release.version),
    ])?;
    run(&["git", "add", "release.yaml"])?;
    run(&["git", "commit", "-m", "chore: cut new nightly release"])?;
    run(&["git", "push"])?;

    Ok(())
}

pub fn release() -> eyre::Result<()> {
    // create version branch
    run(&["git", "checkout", &SOURCE_BRANCH])?;
    run(&["git", "pull"])?;
    let mut release = read_release_file()?;
    let mut version = Version::parse(&release.version)?;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    let blank_version = version.to_string();
    version.pre = Prerelease::new("beta.0")?;
    version.build = BuildMetadata::new("qa")?;
    release.version = version.to_string();
    release.channel = Some(Channel::Qa);
    write_release_file(&release)?;
    run(&["git", "checkout", "-b", &format!("{}{blank_version}", *BRANCH_PREFIX)])?;
    run(&["git", "add", "release.yaml"])?;
    run(&["git", "commit", "-m", "chore: cut new release"])?;
    run(&["git", "push"])?;

    // bump source branch
    run(&["git", "checkout", &SOURCE_BRANCH])?;
    let mut release = read_release_file()?;
    let mut version = Version::parse(&release.version)?;
    version.pre = Prerelease::new("dev")?;
    version.build = BuildMetadata::EMPTY;
    version.patch += 1;
    release.version = version.to_string();
    release.channel = None; // disable package uploads and ci runs
    write_release_file(&release)?;
    run(&["git", "add", "release.yaml"])?;
    run(&["git", "commit", "-m", "chore: bump version after release"])?;
    run(&["git", "push"])?;

    Ok(())
}
