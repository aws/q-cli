pub mod cli;
pub mod daemon;
pub mod dotfiles;
pub mod integrations;
pub mod plugins;
pub mod util;

use clap::StructOpt;

#[tokio::main]
async fn main() {
    // Whitelist init, internal, and tips so those commands do not have sentry
    let _guard = match std::env::args().nth(1).as_deref() {
        Some("init" | "_" | "internal" | "tips") => None,
        _ => fig_telemetry::init_sentry(
            "https://0631fceb9ae540bb874af81820507ebf@o436453.ingest.sentry.io/6187837",
        ),
    };

    cli::Cli::parse().execute().await;
}
