use std::borrow::Cow;
use std::sync::Arc;

use fig_request::reqwest_client::reqwest_client;
use fig_util::Terminal;
pub use sentry::integrations::anyhow::capture_anyhow;
use sentry::transports::ReqwestHttpTransport;
pub use sentry::{
    configure_scope,
    end_session,
    release_name,
    start_session,
};
use sentry::{
    ClientOptions,
    Transport,
};

use crate::util::telemetry_is_disabled;

pub fn init_sentry(
    release: Option<Cow<'static, str>>,
    project: &str,
    sample_rate: f32,
    session_tracking: bool,
) -> Option<sentry::ClientInitGuard> {
    if std::env::var_os("FIG_DISABLE_SENTRY").is_some() {
        None
    } else {
        let guard = sentry::init((project, sentry::ClientOptions {
            release,
            before_send: Some(Arc::new(
                |event| {
                    if telemetry_is_disabled() { None } else { Some(event) }
                },
            )),
            sample_rate,
            auto_session_tracking: session_tracking,
            transport: Some(Arc::new(move |opts: &ClientOptions| -> Arc<dyn Transport> {
                Arc::new(match reqwest_client().cloned() {
                    Some(client) => ReqwestHttpTransport::with_client(opts, client),
                    None => ReqwestHttpTransport::new(opts),
                })
            })),
            ..sentry::ClientOptions::default()
        }));

        sentry::configure_scope(|scope| {
            scope.set_user(Some(sentry::User {
                email: fig_request::auth::get_email(),
                ..sentry::User::default()
            }));

            if let Some(terminal) = Terminal::parent_terminal() {
                scope.set_tag("terminal", terminal.internal_id());
            }

            scope.set_tag("ssh", fig_util::system_info::in_ssh());

            #[cfg(target_os = "linux")]
            scope.set_tag("os.wsl", fig_util::system_info::in_wsl());

            scope.set_tag("fig.version", env!("CARGO_PKG_VERSION"));
        });

        Some(guard)
    }
}
