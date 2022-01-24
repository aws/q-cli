use clap::StructOpt;

pub mod auth;
mod cli;
pub mod config;
pub mod daemon;

#[tokio::main]
async fn main() {
    cli::Cli::parse().execute().await;
}
