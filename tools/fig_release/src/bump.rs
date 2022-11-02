use semver::Prerelease;

use crate::utils::{
    extract_number,
    read_channel,
    read_version,
    run_wet,
    update_lockfile,
    write_version,
    Channel,
};

pub fn bump(dry: bool) -> eyre::Result<()> {
    let mut version = read_version();
    let channel = read_channel();

    match channel {
        Channel::Nightly => eyre::bail!("cannot bump nightly version"),
        Channel::None => eyre::bail!("must have a channel to bump version"),
        Channel::Beta | Channel::Qa => {
            let mut num = extract_number(&version.pre)?;
            num += 1;
            version.pre = Prerelease::new(&format!("beta.{num}"))?;
        },
        Channel::Stable => {
            version.minor += 1;
        },
    }
    write_version(&version);

    update_lockfile()?;

    run_wet(&["git", "add", "Cargo.toml", "Cargo.toml", "Cargo.lock"], dry)?;
    run_wet(
        &[
            "git",
            "commit",
            "-m",
            &format!("chore: bump version to {version} [skip ci]"),
        ],
        dry,
    )?;
    run_wet(&["git", "push"], dry)?;

    Ok(())
}
