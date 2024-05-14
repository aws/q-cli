pub mod cognito;
pub mod endpoint;
mod event;
mod install_method;
mod util;

use std::time::{
    Duration,
    SystemTime,
};

use amzn_codewhisperer_client::types::{
    ChatAddMessageEvent,
    CompletionType,
    IdeCategory,
    OperatingSystem,
    OptOutPreference,
    ProgrammingLanguage,
    TelemetryEvent,
    TerminalUserInteractionEvent,
    TerminalUserInteractionEventType,
    UserContext,
    UserTriggerDecisionEvent,
};
use amzn_toolkit_telemetry::config::{
    AppName,
    BehaviorVersion,
    Region,
};
use amzn_toolkit_telemetry::error::DisplayErrorContext;
use amzn_toolkit_telemetry::types::AwsProduct;
use amzn_toolkit_telemetry::Config;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_smithy_types::DateTime;
use aws_toolkit_telemetry_definitions::IntoMetricDatum;
use cognito::CognitoProvider;
use endpoint::StaticEndpoint;
pub use event::SuggestionState;
use event::{
    Event,
    EventType,
};
use fig_api_client::ai::{
    cw_client,
    cw_endpoint,
};
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
use tokio::sync::{
    Mutex,
    OnceCell,
};
use tokio::task::JoinSet;
use tracing::error;
use util::telemetry_is_disabled;
use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Telemetry is disabled")]
    TelemetryDisabled,
    #[error(transparent)]
    ClientError(#[from] amzn_toolkit_telemetry::operation::post_metrics::PostMetricsError),
}

const APP_NAME: &str = "codewhisperer-terminal";
const PRODUCT: &str = "CodeWhisperer";
const PRODUCT_VERSION: &str = env!("CARGO_PKG_VERSION");

async fn client() -> &'static Client {
    static CLIENT: OnceCell<Client> = OnceCell::const_new();
    CLIENT
        .get_or_init(|| async { Client::new(TelemetryStage::EXTERNAL_PROD).await })
        .await
}

/// A IDE toolkit telemetry stage
///
/// Endpoints from <https://w.amazon.com/bin/view/AWS/DevEx/IDEToolkits/Telemetry/>
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TelemetryStage {
    pub name: &'static str,
    pub endpoint: &'static str,
    pub cognito_pool_id: &'static str,
    pub region: Region,
}

impl TelemetryStage {
    #[allow(dead_code)]
    const BETA: Self = Self::new(
        "beta",
        "https://7zftft3lj2.execute-api.us-east-1.amazonaws.com/Beta",
        "us-east-1:db7bfc9f-8ecd-4fbb-bea7-280c16069a99",
        "us-east-1",
    );
    const EXTERNAL_PROD: Self = Self::new(
        "prod",
        "https://client-telemetry.us-east-1.amazonaws.com",
        "us-east-1:820fd6d1-95c0-4ca4-bffb-3f01d32da842",
        "us-east-1",
    );
    #[allow(dead_code)]
    const INTERNAL_PROD: Self = Self::new(
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

fn opt_out_preference() -> OptOutPreference {
    if telemetry_is_disabled() {
        OptOutPreference::OptOut
    } else {
        OptOutPreference::OptIn
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    client_id: Uuid,
    toolkit_telemetry_client: amzn_toolkit_telemetry::Client,
    codewhisperer_client: amzn_codewhisperer_client::Client,
}

impl Client {
    pub async fn new(telemetry_stage: TelemetryStage) -> Self {
        let client_id = util::get_client_id();
        let toolkit_telemetry_client = amzn_toolkit_telemetry::Client::from_conf(
            Config::builder()
                .behavior_version(BehaviorVersion::v2023_11_09())
                .endpoint_resolver(StaticEndpoint(telemetry_stage.endpoint))
                .app_name(AppName::new(APP_NAME).unwrap())
                .region(telemetry_stage.region.clone())
                .credentials_provider(SharedCredentialsProvider::new(CognitoProvider::new(telemetry_stage)))
                .build(),
        );

        let codewhisperer_client = cw_client(cw_endpoint()).await;

        Self {
            client_id,
            toolkit_telemetry_client,
            codewhisperer_client,
        }
    }

    fn user_context(&self) -> Option<UserContext> {
        let operating_system = match std::env::consts::OS {
            "linux" => OperatingSystem::Linux,
            "macos" => OperatingSystem::Mac,
            "windows" => OperatingSystem::Windows,
            os => {
                error!(%os, "Unsupported operating system");
                return None;
            },
        };

        match UserContext::builder()
            .client_id(self.client_id.hyphenated().to_string())
            .operating_system(operating_system)
            .product(PRODUCT)
            .ide_category(IdeCategory::Cli)
            .ide_version(PRODUCT_VERSION)
            .build()
        {
            Ok(user_context) => Some(user_context),
            Err(err) => {
                error!(%err, "Failed to build user context");
                None
            },
        }
    }

    async fn post_metric(&self, inner: Event) {
        if telemetry_is_disabled() {
            return;
        }

        let toolkit_telemetry_client = self.toolkit_telemetry_client.clone();
        let client_id = self.client_id;

        let mut set = JOIN_SET.lock().await;
        set.spawn({
            let inner = inner.clone();
            async move {
                let inner = inner.into_metric_datum();
                let product = AwsProduct::CodewhispererTerminal;
                let product_version = env!("CARGO_PKG_VERSION");
                let os = std::env::consts::OS;
                let os_architecture = std::env::consts::ARCH;
                let os_version = os_version().map(|v| v.to_string()).unwrap_or_default();
                let metric_name = inner.metric_name().to_owned();

                if let Err(err) = toolkit_telemetry_client
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
            }
        });
    }

    async fn translation_actioned_event(&self, latency: Duration, suggestion_state: SuggestionState) {
        let codewhisperer_client = self.codewhisperer_client.clone();
        let user_context = self.user_context().unwrap();

        let opt_out_preference = opt_out_preference();

        let mut set = JOIN_SET.lock().await;
        set.spawn(async move {
            let mut terminal_user_interaction_event_builder = TerminalUserInteractionEvent::builder()
                .terminal_user_interaction_event_type(
                    TerminalUserInteractionEventType::CodewhispererTerminalTranslationAction,
                )
                .time_to_suggestion(latency.as_millis() as i32)
                .is_completion_accepted(suggestion_state == SuggestionState::Accept);

            if let Some(terminal) = &*CURRENT_TERMINAL {
                terminal_user_interaction_event_builder =
                    terminal_user_interaction_event_builder.terminal(terminal.to_string());
            }

            if let Some(terminal_version) = &*CURRENT_TERMINAL_VERSION {
                terminal_user_interaction_event_builder =
                    terminal_user_interaction_event_builder.terminal_version(terminal_version.to_string());
            }

            if let Some(shell) = Shell::current_shell() {
                terminal_user_interaction_event_builder =
                    terminal_user_interaction_event_builder.shell(shell.to_string());
            }

            let terminal_user_interaction_event = terminal_user_interaction_event_builder.build();

            if let Err(err) = codewhisperer_client
                .send_telemetry_event()
                .telemetry_event(TelemetryEvent::TerminalUserInteractionEvent(
                    terminal_user_interaction_event,
                ))
                .user_context(user_context)
                .opt_out_preference(opt_out_preference)
                .send()
                .await
            {
                error!(err =% DisplayErrorContext(err), "Failed to send telemetry event");
            }
        });
    }

    async fn completion_inserted_event(&self, command: String, terminal: Option<String>, shell: Option<String>) {
        let codewhisperer_client = self.codewhisperer_client.clone();
        let user_context = self.user_context().unwrap();
        let opt_out_preference = opt_out_preference();

        let mut set = JOIN_SET.lock().await;
        set.spawn(async move {
            let mut terminal_user_interaction_event_builder = TerminalUserInteractionEvent::builder()
                .terminal_user_interaction_event_type(
                    TerminalUserInteractionEventType::CodewhispererTerminalCompletionInserted,
                )
                .cli_tool_command(command);

            if let Some(terminal) = terminal {
                terminal_user_interaction_event_builder = terminal_user_interaction_event_builder.terminal(terminal);
            }

            if let Some(shell) = shell {
                terminal_user_interaction_event_builder = terminal_user_interaction_event_builder.shell(shell);
            }

            let terminal_user_interaction_event = terminal_user_interaction_event_builder.build();

            if let Err(err) = codewhisperer_client
                .send_telemetry_event()
                .telemetry_event(TelemetryEvent::TerminalUserInteractionEvent(
                    terminal_user_interaction_event,
                ))
                .user_context(user_context)
                .opt_out_preference(opt_out_preference)
                .send()
                .await
            {
                error!(err =% DisplayErrorContext(err), "Failed to send telemetry event");
            }
        });
    }

    async fn chat_add_message_event(&self, conversation_id: String, message_id: String) {
        let codewhisperer_client = self.codewhisperer_client.clone();
        let user_context = self.user_context().unwrap();
        let opt_out_preference = opt_out_preference();

        let chat_add_message_event = match ChatAddMessageEvent::builder()
            .conversation_id(conversation_id)
            .message_id(message_id)
            .build()
        {
            Ok(event) => event,
            Err(err) => {
                error!(err =% DisplayErrorContext(err), "Failed to send telemetry event");
                return;
            },
        };

        let mut set = JOIN_SET.lock().await;
        set.spawn(async move {
            if let Err(err) = codewhisperer_client
                .send_telemetry_event()
                .telemetry_event(TelemetryEvent::ChatAddMessageEvent(chat_add_message_event))
                .user_context(user_context)
                .opt_out_preference(opt_out_preference)
                .send()
                .await
            {
                error!(err =% DisplayErrorContext(err), "Failed to send telemetry event");
            }
        });
    }

    /// This is the user decision to accept a suggestion for inline suggestions
    async fn user_trigger_decision_event(
        &self,
        session_id: String,
        request_id: String,
        latency: Duration,
        accepted: bool,
    ) {
        let codewhisperer_client = self.codewhisperer_client.clone();
        let user_context = self.user_context().unwrap();
        let opt_out_preference = opt_out_preference();

        let programming_language = match ProgrammingLanguage::builder().language_name("shell").build() {
            Ok(language) => language,
            Err(err) => {
                error!(err =% DisplayErrorContext(err), "Failed to build programming language");
                return;
            },
        };

        let suggestion_state = if accepted {
            SuggestionState::Accept
        } else {
            SuggestionState::Discard
        };

        let user_trigger_decision_event = match UserTriggerDecisionEvent::builder()
            .session_id(session_id)
            .request_id(request_id)
            .programming_language(programming_language)
            .completion_type(CompletionType::Line)
            .suggestion_state(suggestion_state.into())
            .recommendation_latency_milliseconds(latency.as_secs_f64() * 1000.0)
            .timestamp(DateTime::from(SystemTime::now()))
            .build()
        {
            Ok(event) => event,
            Err(err) => {
                error!(err =% DisplayErrorContext(err), "Failed to build user trigger decision event");
                return;
            },
        };

        let mut set = JOIN_SET.lock().await;
        set.spawn(async move {
            if let Err(err) = codewhisperer_client
                .send_telemetry_event()
                .telemetry_event(TelemetryEvent::UserTriggerDecisionEvent(user_trigger_decision_event))
                .user_context(user_context)
                .opt_out_preference(opt_out_preference)
                .send()
                .await
            {
                error!(err =% DisplayErrorContext(err), "Failed to send telemetry event");
            }
        });
    }
}

pub async fn send_user_logged_in() {
    client()
        .await
        .post_metric(Event::new(EventType::UserLoggedIn {}).await)
        .await;
}

pub async fn send_completion_inserted(command: String, terminal: Option<String>, shell: Option<String>) {
    let client = client().await;
    client
        .completion_inserted_event(command.clone(), terminal.clone(), shell.clone())
        .await;

    let event = Event::new(EventType::CompletionInserted {
        command,
        terminal,
        shell,
    })
    .await;

    client.post_metric(event).await;
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineShellCompletionActionedOptions {
    pub session_id: String,
    pub request_id: String,
    pub accepted: bool,
    pub edit_buffer_len: i64,
    pub suggested_chars_len: i64,
    pub latency: Duration,
}

pub async fn send_inline_shell_completion_actioned(options: InlineShellCompletionActionedOptions) {
    let client = client().await;

    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    let event = Event::new(EventType::InlineShellCompletionActioned {
        options: options.clone(),
        terminal: CURRENT_TERMINAL.as_ref().map(|t| t.internal_id().to_string()),
        terminal_version: CURRENT_TERMINAL_VERSION.clone(),
        shell: shell.map(|s| s.to_string()),
        shell_version,
    })
    .await;

    client
        .user_trigger_decision_event(
            options.session_id,
            options.request_id,
            options.latency,
            options.accepted,
        )
        .await;

    client.post_metric(event).await;
}

pub async fn send_translation_actioned(latency: Duration, suggestion_state: SuggestionState) {
    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    let client = client().await;

    let event = Event::new(EventType::TranslationActioned {
        latency,
        suggestion_state,
        terminal: CURRENT_TERMINAL.as_ref().map(|t| t.internal_id().to_string()),
        terminal_version: CURRENT_TERMINAL_VERSION.clone(),
        shell: shell.map(|s| s.to_string()),
        shell_version,
    })
    .await;

    client.translation_actioned_event(latency, suggestion_state).await;

    client.post_metric(event).await;
}

pub async fn send_cli_subcommand_executed(subcommand: impl Into<String>) {
    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    let event = Event::new(EventType::CliSubcommandExecuted {
        subcommand: subcommand.into(),
        terminal: CURRENT_TERMINAL.as_ref().map(|t| t.internal_id().to_string()),
        terminal_version: CURRENT_TERMINAL_VERSION.clone(),
        shell: shell.map(|s| s.to_string()),
        shell_version,
    })
    .await;

    client().await.post_metric(event).await;
}

pub async fn send_doctor_check_failed(failed_check: impl Into<String>) {
    let (shell, shell_version) = Shell::current_shell_version()
        .await
        .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        .unwrap_or((None, None));

    let event = Event::new(EventType::DoctorCheckFailed {
        doctor_check: failed_check.into(),
        terminal: CURRENT_TERMINAL.as_ref().map(|t| t.internal_id().to_string()),
        terminal_version: CURRENT_TERMINAL_VERSION.clone(),
        shell: shell.map(|s| s.to_string()),
        shell_version,
    })
    .await;

    client().await.post_metric(event).await;
}

pub async fn send_dashboard_page_viewed(route: impl Into<String>) {
    let event = Event::new(EventType::DashboardPageViewed { route: route.into() }).await;
    client().await.post_metric(event).await;
}

pub async fn send_menu_bar_actioned(menu_bar_item: Option<impl Into<String>>) {
    let event = Event::new(EventType::MenuBarActioned {
        menu_bar_item: menu_bar_item.map(|i| i.into()),
    })
    .await;
    client().await.post_metric(event).await;
}

pub async fn send_fig_user_migrated() {
    let event = Event::new(EventType::FigUserMigrated {}).await;
    client().await.post_metric(event).await;
}

pub async fn send_start_chat(conversation_id: String) {
    let event = Event::new(EventType::AmazonqStartChat { conversation_id }).await;
    client().await.post_metric(event).await;
}

pub async fn send_end_chat(conversation_id: String) {
    let event = Event::new(EventType::AmazonqEndChat { conversation_id }).await;
    client().await.post_metric(event).await;
}

pub async fn send_chat_added_message(conversation_id: String, message_id: String) {
    client().await.chat_add_message_event(conversation_id, message_id).await;
}

#[cfg(test)]
mod test {
    use fig_util::CLI_BINARY_NAME;
    use uuid::uuid;

    use super::*;

    #[tokio::test]
    async fn client_context() {
        let client = client().await;
        let context = client.user_context().unwrap();

        assert_eq!(context.ide_category, IdeCategory::Cli);
        assert!(matches!(
            context.operating_system,
            OperatingSystem::Linux | OperatingSystem::Mac | OperatingSystem::Windows
        ));
        assert_eq!(context.product, PRODUCT);
        assert_eq!(
            context.client_id,
            Some(uuid!("ffffffff-ffff-ffff-ffff-ffffffffffff").hyphenated().to_string())
        );
        assert_eq!(context.ide_version.as_deref(), Some(PRODUCT_VERSION));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    #[ignore = "needs auth which is not in CI"]
    async fn test_send() {
        // let (shell, shell_version) = Shell::current_shell_version()
        //     .await
        //     .map(|(shell, shell_version)| (Some(shell), Some(shell_version)))
        //     .unwrap_or((None, None));

        // let client = Client::new(TelemetryStage::BETA).await;

        // client
        //     .post_metric(metrics::CodewhispererterminalCliSubcommandExecuted {
        //         create_time: None,
        //         value: None,
        //         codewhispererterminal_subcommand: Some(CodewhispererterminalSubcommand("doctor".into())),
        //         codewhispererterminal_terminal: CURRENT_TERMINAL
        //             .clone()
        //             .map(|terminal| CodewhispererterminalTerminal(terminal.internal_id().to_string())),
        //         codewhispererterminal_terminal_version: CURRENT_TERMINAL_VERSION
        //             .clone()
        //             .map(CodewhispererterminalTerminalVersion),
        //         codewhispererterminal_shell: shell.map(|shell|
        // CodewhispererterminalShell(shell.to_string())),
        //         codewhispererterminal_shell_version:
        // shell_version.map(CodewhispererterminalShellVersion),         credential_start_url:
        // start_url().await,     })
        //     .await;

        finish_telemetry_unwrap().await;

        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
        assert!(!logs_contain("Failed to post metric"));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    #[ignore = "needs auth which is not in CI"]
    async fn test_all_telemetry() {
        send_user_logged_in().await;
        send_completion_inserted(CLI_BINARY_NAME.to_owned(), None, None).await;
        send_inline_shell_completion_actioned(InlineShellCompletionActionedOptions {
            session_id: "".into(),
            request_id: "".into(),
            accepted: true,
            edit_buffer_len: 0,
            suggested_chars_len: 0,
            latency: Duration::from_secs(1),
        })
        .await;
        send_translation_actioned(Duration::from_millis(10), SuggestionState::Accept).await;
        send_cli_subcommand_executed("doctor").await;
        send_doctor_check_failed("").await;
        send_dashboard_page_viewed("/").await;
        send_menu_bar_actioned(Some("Settings")).await;
        send_chat_added_message("debug".to_owned(), "debug".to_owned()).await;

        finish_telemetry_unwrap().await;

        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
        assert!(!logs_contain("Failed to post metric"));
    }

    #[tokio::test]
    #[ignore = "needs auth which is not in CI"]
    async fn test_without_optout() {
        Client::new(TelemetryStage::BETA)
            .await
            .codewhisperer_client
            .send_telemetry_event()
            .telemetry_event(TelemetryEvent::ChatAddMessageEvent(
                ChatAddMessageEvent::builder()
                    .conversation_id("debug".to_owned())
                    .message_id("debug".to_owned())
                    .build()
                    .unwrap(),
            ))
            .send()
            .await
            .unwrap();
    }
}
