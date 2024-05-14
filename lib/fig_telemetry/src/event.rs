use std::time::{
    Duration,
    SystemTime,
};

use aws_toolkit_telemetry_definitions::metrics::{
    AmazonqEndChat,
    AmazonqStartChat,
    CodewhispererterminalCliSubcommandExecuted,
    CodewhispererterminalCompletionInserted,
    CodewhispererterminalDashboardPageViewed,
    CodewhispererterminalDoctorCheckFailed,
    CodewhispererterminalFigUserMigrated,
    CodewhispererterminalInlineShellActioned,
    CodewhispererterminalMenuBarActioned,
    CodewhispererterminalTranslationActioned,
    CodewhispererterminalUserLoggedIn,
};
use aws_toolkit_telemetry_definitions::IntoMetricDatum;

use crate::InlineShellCompletionActionedOptions;

/// A serializable event that can be sent or queued
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Event {
    pub created_time: Option<SystemTime>,
    pub credential_start_url: Option<String>,
    #[serde(flatten)]
    pub ty: EventType,
}

impl Event {
    pub async fn new(ty: EventType) -> Self {
        Self {
            ty,
            credential_start_url: auth::builder_id_token().await.ok().flatten().and_then(|t| t.start_url),
            created_time: Some(SystemTime::now()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SuggestionState {
    Accept,
    Discard,
    Empty,
    Reject,
}

impl SuggestionState {
    fn is_accepted(&self) -> bool {
        matches!(self, SuggestionState::Accept)
    }
}

impl From<SuggestionState> for amzn_codewhisperer_client::types::SuggestionState {
    fn from(value: SuggestionState) -> Self {
        match value {
            SuggestionState::Accept => amzn_codewhisperer_client::types::SuggestionState::Accept,
            SuggestionState::Discard => amzn_codewhisperer_client::types::SuggestionState::Discard,
            SuggestionState::Empty => amzn_codewhisperer_client::types::SuggestionState::Empty,
            SuggestionState::Reject => amzn_codewhisperer_client::types::SuggestionState::Reject,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub(crate) enum EventType {
    UserLoggedIn {},
    CompletionInserted {
        command: String,
        terminal: Option<String>,
        shell: Option<String>,
    },
    InlineShellCompletionActioned {
        #[serde(flatten)]
        options: InlineShellCompletionActionedOptions,
        terminal: Option<String>,
        terminal_version: Option<String>,
        shell: Option<String>,
        shell_version: Option<String>,
    },
    TranslationActioned {
        latency: Duration,
        suggestion_state: SuggestionState,
        terminal: Option<String>,
        terminal_version: Option<String>,
        shell: Option<String>,
        shell_version: Option<String>,
    },
    CliSubcommandExecuted {
        subcommand: String,
        terminal: Option<String>,
        terminal_version: Option<String>,
        shell: Option<String>,
        shell_version: Option<String>,
    },
    DoctorCheckFailed {
        doctor_check: String,
        terminal: Option<String>,
        terminal_version: Option<String>,
        shell: Option<String>,
        shell_version: Option<String>,
    },
    DashboardPageViewed {
        route: String,
    },
    MenuBarActioned {
        menu_bar_item: Option<String>,
    },
    FigUserMigrated {},
    AmazonqStartChat {
        conversation_id: String,
    },
    AmazonqEndChat {
        conversation_id: String,
    },
}

impl IntoMetricDatum for Event {
    fn into_metric_datum(self) -> amzn_toolkit_telemetry::types::MetricDatum {
        match self.ty {
            EventType::UserLoggedIn {} => CodewhispererterminalUserLoggedIn {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
            }
            .into_metric_datum(),
            EventType::CompletionInserted {
                command,
                terminal,
                shell,
            } => CodewhispererterminalCompletionInserted {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                codewhispererterminal_terminal: terminal.map(Into::into),
                codewhispererterminal_terminal_version: None,
                codewhispererterminal_shell: shell.map(Into::into),
                codewhispererterminal_shell_version: None,
                codewhispererterminal_command: Some(command.into()),
                codewhispererterminal_duration: None,
            }
            .into_metric_datum(),
            EventType::InlineShellCompletionActioned {
                options,
                terminal,
                terminal_version,
                shell,
                shell_version,
            } => CodewhispererterminalInlineShellActioned {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                codewhispererterminal_duration: None,
                codewhispererterminal_accepted: Some(options.accepted.into()),
                codewhispererterminal_typed_count: Some(options.edit_buffer_len.into()),
                codewhispererterminal_suggested_count: Some(options.suggested_chars_len.into()),
                codewhispererterminal_terminal: terminal.map(Into::into),
                codewhispererterminal_terminal_version: terminal_version.map(Into::into),
                codewhispererterminal_shell: shell.map(Into::into),
                codewhispererterminal_shell_version: shell_version.map(Into::into),
            }
            .into_metric_datum(),
            EventType::TranslationActioned {
                latency: _,
                suggestion_state,
                terminal,
                terminal_version,
                shell,
                shell_version,
            } => CodewhispererterminalTranslationActioned {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                codewhispererterminal_terminal: terminal.map(Into::into),
                codewhispererterminal_terminal_version: terminal_version.map(Into::into),
                codewhispererterminal_shell: shell.map(Into::into),
                codewhispererterminal_shell_version: shell_version.map(Into::into),
                codewhispererterminal_duration: None,
                codewhispererterminal_time_to_suggestion: None,
                codewhispererterminal_accepted: Some(suggestion_state.is_accepted().into()),
            }
            .into_metric_datum(),
            EventType::CliSubcommandExecuted {
                subcommand,
                terminal,
                terminal_version,
                shell,
                shell_version,
            } => CodewhispererterminalCliSubcommandExecuted {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                codewhispererterminal_terminal: terminal.map(Into::into),
                codewhispererterminal_terminal_version: terminal_version.map(Into::into),
                codewhispererterminal_shell: shell.map(Into::into),
                codewhispererterminal_shell_version: shell_version.map(Into::into),
                codewhispererterminal_subcommand: Some(subcommand.into()),
            }
            .into_metric_datum(),
            EventType::DoctorCheckFailed {
                doctor_check,
                terminal,
                terminal_version,
                shell,
                shell_version,
            } => CodewhispererterminalDoctorCheckFailed {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                codewhispererterminal_terminal: terminal.map(Into::into),
                codewhispererterminal_terminal_version: terminal_version.map(Into::into),
                codewhispererterminal_shell: shell.map(Into::into),
                codewhispererterminal_shell_version: shell_version.map(Into::into),
                codewhispererterminal_doctor_check: Some(doctor_check.into()),
            }
            .into_metric_datum(),
            EventType::DashboardPageViewed { route } => CodewhispererterminalDashboardPageViewed {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                codewhispererterminal_route: Some(route.into()),
            }
            .into_metric_datum(),
            EventType::MenuBarActioned { menu_bar_item } => CodewhispererterminalMenuBarActioned {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                codewhispererterminal_menu_bar_item: menu_bar_item.map(|item| item.into()),
            }
            .into_metric_datum(),
            EventType::FigUserMigrated {} => CodewhispererterminalFigUserMigrated {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
            }
            .into_metric_datum(),
            EventType::AmazonqStartChat { conversation_id } => AmazonqStartChat {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                amazonq_conversation_id: Some(conversation_id.into()),
            }
            .into_metric_datum(),
            EventType::AmazonqEndChat { conversation_id } => AmazonqEndChat {
                create_time: self.created_time,
                value: None,
                credential_start_url: self.credential_start_url.map(Into::into),
                amazonq_conversation_id: Some(conversation_id.into()),
            }
            .into_metric_datum(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn user_logged_in() -> Event {
        Event::new(EventType::UserLoggedIn {}).await
    }

    async fn inline_shell_actioned() -> Event {
        Event::new(EventType::InlineShellCompletionActioned {
            options: InlineShellCompletionActionedOptions {
                session_id: "XXX".into(),
                request_id: "XXX".into(),
                accepted: true,
                edit_buffer_len: 123,
                suggested_chars_len: 42,
                latency: Duration::from_millis(500),
            },
            terminal: Some("vscode".into()),
            terminal_version: Some("1.0".into()),
            shell: Some("bash".into()),
            shell_version: Some("4.4".into()),
        })
        .await
    }
    async fn translation_actioned() -> Event {
        Event::new(EventType::TranslationActioned {
            latency: Duration::from_millis(500),
            suggestion_state: SuggestionState::Accept,
            terminal: Some("vscode".into()),
            terminal_version: Some("1.0".into()),
            shell: Some("bash".into()),
            shell_version: Some("4.4".into()),
        })
        .await
    }

    async fn cli_subcommand_executed() -> Event {
        Event::new(EventType::CliSubcommandExecuted {
            subcommand: "test".into(),
            terminal: Some("vscode".into()),
            terminal_version: Some("1.0".into()),
            shell: Some("bash".into()),
            shell_version: Some("4.4".into()),
        })
        .await
    }

    async fn doctor_check_failed() -> Event {
        Event::new(EventType::DoctorCheckFailed {
            doctor_check: "test".into(),
            terminal: Some("vscode".into()),
            terminal_version: Some("1.0".into()),
            shell: Some("bash".into()),
            shell_version: Some("4.4".into()),
        })
        .await
    }

    async fn dashboard_page_viewed() -> Event {
        Event::new(EventType::DashboardPageViewed { route: "test".into() }).await
    }

    async fn menu_bar_actioned() -> Event {
        Event::new(EventType::MenuBarActioned {
            menu_bar_item: Some("test".into()),
        })
        .await
    }

    async fn fig_user_migrated() -> Event {
        Event::new(EventType::FigUserMigrated {}).await
    }

    async fn amazonq_start_chat() -> Event {
        Event::new(EventType::AmazonqStartChat {
            conversation_id: "XXX".into(),
        })
        .await
    }

    async fn amazonq_end_chat() -> Event {
        Event::new(EventType::AmazonqEndChat {
            conversation_id: "XXX".into(),
        })
        .await
    }

    async fn all_events() -> Vec<Event> {
        vec![
            user_logged_in().await,
            inline_shell_actioned().await,
            translation_actioned().await,
            cli_subcommand_executed().await,
            doctor_check_failed().await,
            dashboard_page_viewed().await,
            menu_bar_actioned().await,
            fig_user_migrated().await,
            amazonq_start_chat().await,
            amazonq_end_chat().await,
        ]
    }

    #[tokio::test]
    async fn test_event_ser() {
        for event in all_events().await {
            let json = serde_json::to_string_pretty(&event).unwrap();
            println!("\n{json}\n");
            let deser: Event = serde_json::from_str(&json).unwrap();
            assert_eq!(event, deser);
        }
    }

    #[tokio::test]
    async fn test_into_metric_datum() {
        for event in all_events().await {
            let metric_datum = event.into_metric_datum();
            println!("\n{}: {metric_datum:?}\n", metric_datum.metric_name());
        }
    }
}
