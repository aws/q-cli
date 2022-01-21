use clap::StructOpt;

pub mod auth;
mod cli;
pub mod config;

#[tokio::main]
async fn main() {
    cli::Cli::parse().execute().await;
}
