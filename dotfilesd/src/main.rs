use clap::StructOpt;

pub mod auth;
pub mod cli;
pub mod config;
pub mod daemon;
pub mod plugins;
pub mod proto;
pub mod ipc;
pub mod util;

#[tokio::main]
async fn main() {
    cli::Cli::parse().execute().await;
}
