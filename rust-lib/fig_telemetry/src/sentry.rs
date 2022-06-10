use std::sync::Arc;

use fig_util::Terminal;

use crate::util::telemetry_is_disabled;

pub fn init_sentry(project: &str) -> Option<sentry::ClientInitGuard> {
    if std::env::var_os("FIG_DISABLE_SENTRY").is_some() {
        None
    } else {
        let guard = sentry::init((project, sentry::ClientOptions {
            release: sentry::release_name!(),
            before_send: Some(Arc::new(
                |event| {
                    if telemetry_is_disabled() { None } else { Some(event) }
                },
            )),
            ..sentry::ClientOptions::default()
        }));

        #[cfg(target_os = "macos")]
        let terminal = Terminal::parent_terminal().map(|s| s.to_string());
        #[cfg(not(target_os = "macos"))]
        let terminal: Option<Terminal> = None;

        sentry::configure_scope(|scope| {
            scope.set_user(Some(sentry::User {
                email: fig_auth::get_email(),
                ..sentry::User::default()
            }));

            if let Some(terminal) = terminal {
                scope.set_tag("terminal", terminal);
            }
        });

        Some(guard)
    }
}

pub fn capture_anyhow(e: &anyhow::Error) {
    sentry::integrations::anyhow::capture_anyhow(e);
}
