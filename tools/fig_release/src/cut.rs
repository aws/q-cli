use fig_settings::state;
use semver::{
    BuildMetadata,
    Prerelease,
};

use crate::utils::{
    gen_nightly,
    read_version,
    run_wet,
    update_lockfile,
    write_channel,
    write_version,
    Channel,
};
fn source_branch() -> String {
    state::get_string("developer.release.sourceBranch")
        .ok()
        .flatten()
        .unwrap_or_else(|| "develop".into())
}

fn branch_prefix() -> String {
    state::get_string("developer.release.branchPrefix")
        .ok()
        .flatten()
        .unwrap_or_else(|| "".into())
}

pub fn nightly(dry: bool) -> eyre::Result<()> {
    run_wet(&["git", "checkout", &source_branch()], dry)?;
    run_wet(&["git", "pull"], dry)?;
    let mut version = read_version();
    version.pre = Prerelease::new(&format!("nightly.{}", gen_nightly()))?;
    version.build = BuildMetadata::EMPTY;
    write_version(&version);
    write_channel(&Channel::Nightly);
    update_lockfile()?;
    run_wet(
        &["git", "checkout", "-b", &format!("{}{}", branch_prefix(), version)],
        dry,
    )?;
    run_wet(&["git", "add", "Cargo.toml", "Cargo.lock"], dry)?;
    run_wet(&["git", "commit", "-m", "chore: cut new nightly release"], dry)?;
    run_wet(&["git", "push"], dry)?;

    Ok(())
}

pub fn release(dry: bool) -> eyre::Result<()> {
    let source_branch = source_branch();

    // create version branch
    run_wet(&["git", "checkout", &source_branch], dry)?;
    run_wet(&["git", "pull"], dry)?;
    let mut version = read_version();
    version.pre = Prerelease::EMPTY;
    version.build = BuildMetadata::EMPTY;
    let blank_version = version.to_string();
    version.pre = Prerelease::new("beta.0")?;
    version.build = BuildMetadata::new("qa")?;
    write_version(&version);
    write_channel(&Channel::Qa);
    update_lockfile()?;
    run_wet(
        &["git", "checkout", "-b", &format!("{}{blank_version}", branch_prefix())],
        dry,
    )?;
    run_wet(&["git", "add", "Cargo.toml", "Cargo.lock"], dry)?;
    run_wet(&["git", "commit", "-m", "chore: cut new release"], dry)?;
    run_wet(&["git", "push"], dry)?;

    // bump source branch
    run_wet(&["git", "checkout", &source_branch], dry)?;
    let mut version = read_version();
    version.pre = Prerelease::new("dev")?;
    version.build = BuildMetadata::EMPTY;
    version.patch += 1;
    write_version(&version);
    write_channel(&Channel::None); // disable package uploads and ci runs
    update_lockfile()?;
    run_wet(&["git", "add", "Cargo.toml", "Cargo.lock"], dry)?;
    run_wet(&["git", "commit", "-m", "chore: bump version after release"], dry)?;
    run_wet(&["git", "push"], dry)?;

    Ok(())
}
