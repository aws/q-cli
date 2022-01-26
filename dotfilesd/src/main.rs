use clap::StructOpt;

pub mod auth;
pub mod cli;
pub mod daemon;
pub mod util;
pub mod config;

#[tokio::main]
async fn main() {
    cli::Cli::parse().execute().await;
}
