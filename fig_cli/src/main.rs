#[macro_use]
extern crate cfg_if;

pub mod cli;
pub mod daemon;
pub mod dotfiles;
pub mod integrations;
pub mod plugins;
pub mod util;

use clap::StructOpt;
use fig_auth::get_email;

#[tokio::main]
async fn main() {
    // Whitelist init, internal, and tips so those commands do not have sentry
    let _guard = match std::env::args().nth(1).as_deref() {
        Some("init" | "_" | "internal" | "tips") => None,
        _ => {
            if std::env::var_os("FIG_DISABLE_SENTRY").is_some() {
                None
            } else {
                let guard = sentry::init((
                    "https://0631fceb9ae540bb874af81820507ebf@o436453.ingest.sentry.io/6187837",
                    sentry::ClientOptions {
                        release: sentry::release_name!(),
                        ..sentry::ClientOptions::default()
                    },
                ));

                sentry::configure_scope(|scope| {
                    scope.set_user(Some(sentry::User {
                        email: get_email(),
                        ..sentry::User::default()
                    }));
                });

                Some(guard)
            }
        }
    };

    cli::Cli::parse().execute().await;
}
