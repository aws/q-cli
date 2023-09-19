use semver::{
    BuildMetadata,
    Prerelease,
};

use crate::utils::{
    extract_number,
    read_version,
    run_wet,
    update_lockfile,
    write_channel,
    write_version,
    Channel,
};

pub fn beta(dry: bool) -> eyre::Result<()> {
    run_wet(&["git", "pull"], dry)?;
    let mut version = read_version();
    let mut num = extract_number(&version.pre)?;
    num += 1;
    version.pre = Prerelease::new(&format!("beta.{num}"))?;
    version.build = BuildMetadata::EMPTY;
    write_version(&version);
    write_channel(&Channel::Beta);
    update_lockfile()?;
    run_wet(&["git", "add", "Cargo.toml", "Cargo.lock"], dry)?;
    run_wet(&["git", "commit", "-m", "chore: promote qa to beta"], dry)?;
    run_wet(&["git", "push"], dry)?;

    Ok(())
}

pub fn stable(dry: bool) -> eyre::Result<()> {
    run_wet(&["git", "pull"], dry)?;
    let mut version = read_version();
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    write_version(&version);
    write_channel(&Channel::Stable);
    update_lockfile()?;
    run_wet(&["git", "add", "Cargo.toml", "Cargo.lock"], dry)?;
    run_wet(&["git", "commit", "-m", "chore: promote beta to stable"], dry)?;
    run_wet(&["git", "push"], dry)?;

    Ok(())
}
