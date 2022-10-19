use semver::{
    Prerelease,
    Version,
};

use crate::utils::{
    extract_number,
    read_release_file,
    run,
    sync_version,
    write_release_file,
    Channel,
};

pub fn bump() -> eyre::Result<()> {
    let mut release = read_release_file()?;
    match release.channel {
        Some(Channel::Nightly) => eyre::bail!("cannot bump nightly version"),
        Some(Channel::Stable) => eyre::bail!("cannot bump stable version"),
        Some(_) => {},
        None => eyre::bail!("must have a channel to bump version"),
    }
    let mut version = Version::parse(&release.version)?;
    let mut num = extract_number(&version.pre)?;
    num += 1;
    version.pre = Prerelease::new(&format!("beta.{num}"))?;
    release.version = version.to_string();
    write_release_file(&release)?;

    sync_version(&release)?;

    run(&["cargo", "update", "--offline", "--workspace"])?;

    run(&["git", "add", "release.yaml", "Cargo.toml", "Cargo.lock"])?;
    run(&["git", "commit", "-m", "chore: bump version"])?;
    run(&["git", "push"])?;

    Ok(())
}
