pub mod cli;
pub mod daemon;
pub mod dotfiles;
pub mod plugins;
pub mod util;

use clap::StructOpt;
use fig_auth::get_email;

#[tokio::main]
async fn main() {
    let _guard = sentry::init((
        "https://0631fceb9ae540bb874af81820507ebf@o436453.ingest.sentry.io/6187837",
        sentry::ClientOptions {
            release: sentry::release_name!(),
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_user(Some(sentry::User {
            email: get_email(),
            ..Default::default()
        }));
    });

    cli::Cli::parse().execute().await;
}
