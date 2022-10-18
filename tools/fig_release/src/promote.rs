use semver::{
    BuildMetadata,
    Prerelease,
    Version,
};

use crate::utils::{
    extract_number,
    read_release_file,
    run,
    write_release_file,
    Channel,
};

pub fn beta() -> eyre::Result<()> {
    run(&["git", "pull"])?;
    let mut release = read_release_file()?;
    let mut version = Version::parse(&release.version)?;
    let mut num = extract_number(&version.pre)?;
    num += 1;
    version.pre = Prerelease::new(&format!("beta.{num}"))?;
    version.build = BuildMetadata::EMPTY;
    release.version = version.to_string();
    release.channel = Some(Channel::Beta);
    write_release_file(&release)?;
    run(&["git", "add", "release.yaml"])?;
    run(&["git", "commit", "-m", "chore: promote qa to beta"])?;
    run(&["git", "push"])?;

    Ok(())
}

pub fn stable() -> eyre::Result<()> {
    run(&["git", "pull"])?;
    let mut release = read_release_file()?;
    let mut version = Version::parse(&release.version)?;
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    release.version = version.to_string();
    release.channel = Some(Channel::Stable);
    write_release_file(&release)?;
    run(&["git", "add", "release.yaml"])?;
    run(&["git", "commit", "-m", "chore: promote beta to stable"])?;
    run(&["git", "push"])?;

    Ok(())
}
