use clap::StructOpt;

mod cli;
pub mod config;

#[tokio::main]
async fn main() {
    cli::Cli::parse().execute().await;
}
