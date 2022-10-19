use clap::Parser;
use cli::{
    Cut,
    Promote,
};

use crate::cli::{
    ChangelogAction,
    Cli,
};

mod bump;
mod changelog;
mod cli;
mod cut;
mod debug;
mod package;
mod promote;
mod publish;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    match Cli::parse().subcommand {
        cli::Sub::Changelog {
            action: ChangelogAction::Add,
        } => changelog::add()?,
        cli::Sub::Changelog {
            action: ChangelogAction::Edit,
        } => changelog::edit()?,
        cli::Sub::Changelog {
            action: ChangelogAction::Remove,
        } => changelog::remove()?,
        cli::Sub::Cut { channel: Cut::Nightly } => cut::nightly()?,
        cli::Sub::Cut { channel: Cut::Release } => cut::release()?,
        cli::Sub::Promote { channel: Promote::Beta } => promote::beta()?,
        cli::Sub::Promote {
            channel: Promote::Stable,
        } => promote::stable()?,
        cli::Sub::Package {
            path,
            kind,
            architecture,
            variant,
        } => package::package(path, kind, architecture, variant).await?,
        cli::Sub::Bump => bump::bump()?,
        cli::Sub::Debug { action } => debug::debug(action).await?,
        cli::Sub::Publish { build_targets } => publish::publish(build_targets).await?,
    }
    Ok(())
}
