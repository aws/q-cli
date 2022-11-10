use clap::Parser;
use cli::{
    Cut,
    Promote,
};

use crate::cli::Cli;

mod bump;
// mod changelog;
mod cli;
mod cut;
mod debug;
mod package;
mod promote;
mod publish;
mod utils;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = Cli::parse();
    match cli.subcommand {
        // cli::Sub::Changelog {
        //     action: ChangelogAction::Add,
        // } => changelog::add()?,
        // cli::Sub::Changelog {
        //     action: ChangelogAction::Edit,
        // } => changelog::edit()?,
        // cli::Sub::Changelog {
        //     action: ChangelogAction::Remove,
        // } => changelog::remove()?,
        cli::Sub::Cut { channel: Cut::Nightly } => cut::nightly(cli.dry)?,
        cli::Sub::Cut { channel: Cut::Release } => cut::release(cli.dry)?,
        cli::Sub::Promote { channel: Promote::Beta } => promote::beta(cli.dry)?,
        cli::Sub::Promote {
            channel: Promote::Stable,
        } => promote::stable(cli.dry)?,
        cli::Sub::Package {
            path,
            kind,
            architecture,
            variant,
        } => package::package(path, kind, architecture, variant, cli.dry).await?,
        cli::Sub::Bump => bump::bump(cli.dry)?,
        cli::Sub::Debug { action } => debug::debug(action).await?,
        cli::Sub::Publish { build_targets } => publish::publish(build_targets, cli.dry, cli.yes).await?,
        _ => todo!(),
    }
    Ok(())
}
