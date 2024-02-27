pub mod cognito;
pub mod endpoint;
mod install_method;
mod util;

use std::time::SystemTime;

use amzn_toolkit_telemetry::config::{
    AppName,
    BehaviorVersion,
    Region,
};
use amzn_toolkit_telemetry::error::DisplayErrorContext;
use amzn_toolkit_telemetry::types::AwsProduct;
use amzn_toolkit_telemetry::{
    Client,
    Config,
};
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_toolkit_telemetry_definitions::metrics::{
    CodewhispererterminalCliSubcommandExecuted,
    CodewhispererterminalDashboardPageViewed,
    CodewhispererterminalDoctorCheckFailed,
    CodewhispererterminalFigUserMigrated,
    CodewhispererterminalMenuBarActioned,
    CodewhispererterminalTranslationActioned,
};
use aws_toolkit_telemetry_definitions::types::{
    CodewhispererterminalAccepted,
    CodewhispererterminalCommand,
    CodewhispererterminalDoctorCheck,
    CodewhispererterminalMenuBarItem,
    CodewhispererterminalRoute,
    CodewhispererterminalShell,
    CodewhispererterminalShellVersion,
    CodewhispererterminalSubcommand,
    CodewhispererterminalSuggestedCount,
    CodewhispererterminalTerminal,
    CodewhispererterminalTerminalVersion,
    CodewhispererterminalTypedCount,
    CredentialStartUrl,
};
use aws_toolkit_telemetry_definitions::{
    metrics,
    IntoMetricDatum,
};
use cognito::CognitoProvider;
use endpoint::StaticEndpoint;
use fig_util::system_info::os_version;
use fig_util::terminal::{
    CURRENT_TERMINAL,
    CURRENT_TERMINAL_VERSION,
};
use fig_util::Shell;
pub use install_method::{
    get_install_method,
    InstallMethod,
};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::error;
use util::telemetry_is_disabled;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("telemetry is disabled")]
    TelemetryDisabled,
    #[error(transparent)]
    ClientError(#[from] amzn_toolkit_telemetry::operation::post_metrics::PostMetricsError),
}

const APP_NAME: &str = "codewhisperer-terminal";

static CLIENT: Lazy<TelemetryClient> = Lazy::new(|| TelemetryClient::new(TelemetryStage::EXTERNAL_PROD));

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TelemetryStage {
    pub name: &'static str,
    pub endpoint: &'static str,
    pub cognito_pool_id: &'static str,
    pub region: Region,
}

impl TelemetryStage {
    // data from <https://w.amazon.com/bin/view/AWS/DevEx/IDEToolkits/Telemetry/>
    pub const BETA: Self = Self::new(
        "beta",
        "https://7zftft3lj2.execute-api.us-east-1.amazonaws.com/Beta",
        "us-east-1:db7bfc9f-8ecd-4fbb-bea7-280c16069a99",
        "us-east-1",
    );
    pub const EXTERNAL_PROD: Self = Self::new(
        "prod",
        "https://client-telemetry.us-east-1.amazonaws.com",
        "us-east-1:820fd6d1-95c0-4ca4-bffb-3f01d32da842",
        "us-east-1",
    );
    pub const INTERNAL_PROD: Self = Self::new(
        "internal-prod",
        "https://1ek5zo40ci.execute-api.us-east-1.amazonaws.com/InternalProd",
        "us-east-1:4037bda8-adbd-4c71-ae5e-88b270261c25",
        "us-east-1",
    );

    const fn new(
        name: &'static str,
        endpoint: &'static str,
        cognito_pool_id: &'static str,
        region: &'static str,
    ) -> Self {
        Self {
            name,
            endpoint,
            cognito_pool_id,
            region: Region::from_static(region),
        }
    }
}

static JOIN_SET: Lazy<Mutex<JoinSet<()>>> = Lazy::new(|| Mutex::new(JoinSet::new()));

/// Joins all current telemetry events
pub async fn finish_telemetry() {
    let mut set = JOIN_SET.lock().await;
    while let Some(res) = set.join_next().await {
        if let Err(err) = res {
            error!(%err, "Failed to join telemetry event");
        }
    }
}

/// Joins all current telemetry events and panics if any fail to join
pub async fn finish_telemetry_unwrap() {
    let mut set = JOIN_SET.lock().await;
    while let Some(res) = set.join_next().await {
        res.unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryClient {
    client_id: Uuid,
    aws_client: Client,
}

impl TelemetryClient {
    pub fn new(telemetry_stage: TelemetryStage) -> Self {
        let client_id = util::get_client_id();
        let aws_client = Client::from_conf(
            Config::builder()
                .behavior_version(BehaviorVersion::v2023_11_09())
                .endpoint_resolver(StaticEndpoint(telemetry_stage.endpoint))
                .app_name(AppName::new(APP_NAME).unwrap())
                .region(telemetry_stage.region.clone())
                .credentials_provider(SharedCredentialsProvider::new(CognitoProvider::new(telemetry_stage)))
                .build(),
        );
        Self { client_id, aws_client }
    }

    pub async fn post_metric(&self, inner: impl IntoMetricDatum + 'static) {
        if telemetry_is_disabled() {
            return;
        }

        let aws_client = self.aws_client.clone();
        let client_id = self.client_id;

        let mut set = JOIN_SET.lock().await;
        set.spawn(async move {
            let inner = inner.into_metric_datum();
            let product = AwsProduct::CodewhispererTerminal;
            let product_version = env!("CARGO_PKG_VERSION");
            let os = std::env::consts::OS;
            let os_architecture = std::env::consts::ARCH;
            let os_version = os_version().map(|v| v.to_string()).unwrap_or_default();
            let metric_name = inner.metric_name().to_owned();

            if let Err(err) = aws_client
                .post_metrics()
                .aws_product(product)
                .aws_product_version(product_version)
                .client_id(client_id)
                .os(os)
                .os_architecture(os_architecture)
                .os_version(os_version)
                .metric_data(inner)
                .send()
                .await
                .map_err(DisplayErrorContext)
            {
                error!(%err, ?metric_name, "Failed to post metric");
            }
        });
    }
}

async fn start_url() -> Option<CredentialStartUrl> {
    auth::builder_id_token()
        .await
        .ok()
        .flatten()
        .and_then(|t| t.start_url)
        .map(CredentialStartUrl)
}

pub async fn send_user_logged_in() {
    CLIENT
        .post_metric(metrics::CodewhispererterminalUserLoggedIn {
            create_time: Some(SystemTime::now()),
            value: None,
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_completion_inserted(command: String, terminal: Option<String>, shell: Option<String>) {
    CLIENT
        .post_metric(metrics::CodewhispererterminalCompletionInserted {
            create_time: None,
            value: None,
            codewhispererterminal_command: Some(CodewhispererterminalCommand(command)),
            codewhispererterminal_duration: None,
            codewhispererterminal_terminal: terminal.map(CodewhispererterminalTerminal),
            codewhispererterminal_terminal_version: None,
            codewhispererterminal_shell: shell.map(CodewhispererterminalShell),
            codewhispererterminal_shell_version: None,
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_ghost_text_actioned(accepted: bool, edit_buffer_len: usize, suggested_chars_len: usize) {
    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    CLIENT
        .post_metric(metrics::CodewhispererterminalGhostTextActioned {
            create_time: None,
            value: None,
            codewhispererterminal_duration: None,
            codewhispererterminal_accepted: Some(CodewhispererterminalAccepted(accepted)),
            codewhispererterminal_typed_count: Some(CodewhispererterminalTypedCount(edit_buffer_len as i64)),
            codewhispererterminal_suggested_count: Some(CodewhispererterminalSuggestedCount(
                suggested_chars_len as i64,
            )),
            codewhispererterminal_terminal: CURRENT_TERMINAL
                .clone()
                .map(|terminal| CodewhispererterminalTerminal(terminal.internal_id().to_string())),
            codewhispererterminal_terminal_version: CURRENT_TERMINAL_VERSION
                .clone()
                .map(CodewhispererterminalTerminalVersion),
            codewhispererterminal_shell: shell.map(|shell| CodewhispererterminalShell(shell.to_string())),
            codewhispererterminal_shell_version: shell_version.map(CodewhispererterminalShellVersion),
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_translation_actioned(
    // time_viewed: i64,
    // time_waited: i64,
    accepted: bool,
) {
    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    CLIENT
        .post_metric(CodewhispererterminalTranslationActioned {
            create_time: None,
            value: None,
            codewhispererterminal_duration: None,
            codewhispererterminal_time_to_suggestion: None,
            codewhispererterminal_accepted: Some(CodewhispererterminalAccepted(accepted)),
            codewhispererterminal_terminal: CURRENT_TERMINAL
                .clone()
                .map(|terminal| CodewhispererterminalTerminal(terminal.internal_id().to_string())),
            codewhispererterminal_terminal_version: CURRENT_TERMINAL_VERSION
                .clone()
                .map(CodewhispererterminalTerminalVersion),
            codewhispererterminal_shell: shell.map(|shell| CodewhispererterminalShell(shell.to_string())),
            codewhispererterminal_shell_version: shell_version.map(CodewhispererterminalShellVersion),
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_cli_subcommand_executed(command_name: impl Into<String>) {
    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    CLIENT
        .post_metric(CodewhispererterminalCliSubcommandExecuted {
            create_time: None,
            value: None,
            codewhispererterminal_subcommand: Some(CodewhispererterminalSubcommand(command_name.into())),
            codewhispererterminal_terminal: CURRENT_TERMINAL
                .clone()
                .map(|terminal| CodewhispererterminalTerminal(terminal.internal_id().to_string())),
            codewhispererterminal_terminal_version: CURRENT_TERMINAL_VERSION
                .clone()
                .map(CodewhispererterminalTerminalVersion),
            codewhispererterminal_shell: shell.map(|shell| CodewhispererterminalShell(shell.to_string())),
            codewhispererterminal_shell_version: shell_version.map(CodewhispererterminalShellVersion),
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_doctor_check_failed(failed_check: impl Into<String>) {
    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    CLIENT
        .post_metric(CodewhispererterminalDoctorCheckFailed {
            create_time: None,
            value: None,
            codewhispererterminal_doctor_check: Some(CodewhispererterminalDoctorCheck(failed_check.into())),
            codewhispererterminal_terminal: CURRENT_TERMINAL
                .clone()
                .map(|terminal| CodewhispererterminalTerminal(terminal.internal_id().to_string())),
            codewhispererterminal_terminal_version: CURRENT_TERMINAL_VERSION
                .clone()
                .map(CodewhispererterminalTerminalVersion),
            codewhispererterminal_shell: shell.map(|shell| CodewhispererterminalShell(shell.to_string())),
            codewhispererterminal_shell_version: shell_version.map(CodewhispererterminalShellVersion),
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_dashboard_page_viewed(route: impl Into<String>) {
    CLIENT
        .post_metric(CodewhispererterminalDashboardPageViewed {
            create_time: None,
            value: None,
            codewhispererterminal_route: Some(CodewhispererterminalRoute(route.into())),
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_menu_bar_actioned(menu_bar_item: Option<impl Into<String>>) {
    CLIENT
        .post_metric(CodewhispererterminalMenuBarActioned {
            create_time: None,
            value: None,
            codewhispererterminal_menu_bar_item: menu_bar_item
                .map(|item| CodewhispererterminalMenuBarItem(item.into())),
            credential_start_url: start_url().await,
        })
        .await;
}

pub async fn send_fig_user_migrated() {
    CLIENT
        .post_metric(CodewhispererterminalFigUserMigrated {
            create_time: None,
            value: None,
            credential_start_url: start_url().await,
        })
        .await;
}

#[cfg(test)]
mod test {
    use super::*;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_send() {
        let (shell, shell_version) = Shell::current_shell_version()
            .await
            .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
            .unwrap_or((None, None));

        let client = TelemetryClient::new(TelemetryStage::BETA);

        client
            .post_metric(metrics::CodewhispererterminalCliSubcommandExecuted {
                create_time: None,
                value: None,
                codewhispererterminal_subcommand: Some(CodewhispererterminalSubcommand("doctor".into())),
                codewhispererterminal_terminal: CURRENT_TERMINAL
                    .clone()
                    .map(|terminal| CodewhispererterminalTerminal(terminal.internal_id().to_string())),
                codewhispererterminal_terminal_version: CURRENT_TERMINAL_VERSION
                    .clone()
                    .map(CodewhispererterminalTerminalVersion),
                codewhispererterminal_shell: shell.map(|shell| CodewhispererterminalShell(shell.to_string())),
                codewhispererterminal_shell_version: shell_version.map(CodewhispererterminalShellVersion),
                credential_start_url: start_url().await,
            })
            .await;

        finish_telemetry_unwrap().await;

        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
        assert!(!logs_contain("Failed to post metric"));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_all_telemetry() {
        send_user_logged_in().await;
        send_completion_inserted("cw".to_owned(), None, None).await;
        send_ghost_text_actioned(true, 1, 2).await;
        send_translation_actioned(true).await;
        send_cli_subcommand_executed("doctor").await;
        send_doctor_check_failed("").await;
        send_dashboard_page_viewed("/").await;
        send_menu_bar_actioned(Some("Settings")).await;

        finish_telemetry_unwrap().await;

        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
        assert!(!logs_contain("Failed to post metric"));
    }
}
