use semver::{
    BuildMetadata,
    Prerelease,
};

use crate::cli::BumpTo;
use crate::utils::{
    extract_number,
    read_channel,
    read_version,
    run_wet,
    update_lockfile,
    write_channel,
    write_version,
    Channel,
};

pub fn bump(dry: bool, to: Option<BumpTo>) -> eyre::Result<()> {
    match to {
        Some(to) => bump_nonnormal(dry, to),
        None => bump_normal(dry),
    }
}

fn bump_normal(dry: bool) -> eyre::Result<()> {
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
            version.patch += 1;
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

pub fn bump_nonnormal(dry: bool, to: BumpTo) -> eyre::Result<()> {
    if dry {
        eyre::bail!("All cross-channel bumps are dry by default");
    }

    let channel = read_channel();

    match (&channel, &to) {
        (Channel::Nightly, _) => eyre::bail!("Can't cross-channel bump from nightly"),
        (Channel::None, _) => eyre::bail!("Can't cross-channel bump without a channel"),
        (Channel::Qa, &BumpTo::Qa) | (Channel::Beta, BumpTo::Beta) => {
            eyre::bail!("Can't cross-channel bump to the current channel")
        },
        (Channel::Stable, &BumpTo::Beta)
        | (Channel::Stable, &BumpTo::Qa)
        | (Channel::Qa, &BumpTo::Beta)
        | (Channel::Beta, BumpTo::Qa) => {},
    }

    if !dialoguer::Confirm::new()
        .with_prompt("Are you sure you want to perform a cross-channel bump? This change may break the release cycle!")
        .interact()?
    {
        eyre::bail!("Cancelled");
    }

    bump_normal(true)?;

    let mut version = read_version();

    if channel == Channel::Stable {
        version.pre = Prerelease::new("beta.0")?;
    }

    match to {
        BumpTo::Qa => {
            version.build = BuildMetadata::new("qa")?;
            write_channel(&Channel::Qa);
        },
        BumpTo::Beta => {
            version.build = BuildMetadata::EMPTY;
            write_channel(&Channel::Beta);
        },
    }
    write_version(&version);
    update_lockfile()?;

    Ok(())
}
