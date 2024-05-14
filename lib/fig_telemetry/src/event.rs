use std::time::{
    Duration,
    SystemTime,
};

use amzn_toolkit_telemetry::types::MetricDatum;
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

/// A serializable event that can be sent or queued
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
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

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
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
    pub(crate) fn is_accepted(&self) -> bool {
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
pub struct InlineShellCompletionActionedOptions {}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum EventType {
    UserLoggedIn {},
    CompletionInserted {
        command: String,
        terminal: Option<String>,
        shell: Option<String>,
    },
    InlineShellCompletionActioned {
        session_id: String,
        request_id: String,
        suggestion_state: SuggestionState,
        edit_buffer_len: Option<i64>,
        suggested_chars_len: Option<i64>,
        latency: Duration,
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
    ChatStart {
        conversation_id: String,
    },
    ChatEnd {
        conversation_id: String,
    },
    ChatAddedMessage {
        conversation_id: String,
        message_id: String,
    },
}

impl Event {
    pub(crate) fn into_metric_datum(self) -> Option<MetricDatum> {
        match self.ty {
            EventType::UserLoggedIn {} => Some(
                CodewhispererterminalUserLoggedIn {
                    create_time: self.created_time,
                    value: None,
                    credential_start_url: self.credential_start_url.map(Into::into),
                }
                .into_metric_datum(),
            ),
            EventType::CompletionInserted {
                command,
                terminal,
                shell,
            } => Some(
                CodewhispererterminalCompletionInserted {
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
            ),
            EventType::InlineShellCompletionActioned {
                terminal,
                terminal_version,
                shell,
                shell_version,
                suggestion_state,
                edit_buffer_len,
                suggested_chars_len,
                ..
            } => Some(
                CodewhispererterminalInlineShellActioned {
                    create_time: self.created_time,
                    value: None,
                    credential_start_url: self.credential_start_url.map(Into::into),
                    codewhispererterminal_duration: None,
                    codewhispererterminal_accepted: Some(suggestion_state.is_accepted().into()),
                    codewhispererterminal_typed_count: edit_buffer_len.map(Into::into),
                    codewhispererterminal_suggested_count: suggested_chars_len.map(Into::into),
                    codewhispererterminal_terminal: terminal.map(Into::into),
                    codewhispererterminal_terminal_version: terminal_version.map(Into::into),
                    codewhispererterminal_shell: shell.map(Into::into),
                    codewhispererterminal_shell_version: shell_version.map(Into::into),
                }
                .into_metric_datum(),
            ),
            EventType::TranslationActioned {
                latency: _,
                suggestion_state,
                terminal,
                terminal_version,
                shell,
                shell_version,
            } => Some(
                CodewhispererterminalTranslationActioned {
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
            ),
            EventType::CliSubcommandExecuted {
                subcommand,
                terminal,
                terminal_version,
                shell,
                shell_version,
            } => Some(
                CodewhispererterminalCliSubcommandExecuted {
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
            ),
            EventType::DoctorCheckFailed {
                doctor_check,
                terminal,
                terminal_version,
                shell,
                shell_version,
            } => Some(
                CodewhispererterminalDoctorCheckFailed {
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
            ),
            EventType::DashboardPageViewed { route } => Some(
                CodewhispererterminalDashboardPageViewed {
                    create_time: self.created_time,
                    value: None,
                    credential_start_url: self.credential_start_url.map(Into::into),
                    codewhispererterminal_route: Some(route.into()),
                }
                .into_metric_datum(),
            ),
            EventType::MenuBarActioned { menu_bar_item } => Some(
                CodewhispererterminalMenuBarActioned {
                    create_time: self.created_time,
                    value: None,
                    credential_start_url: self.credential_start_url.map(Into::into),
                    codewhispererterminal_menu_bar_item: menu_bar_item.map(|item| item.into()),
                }
                .into_metric_datum(),
            ),
            EventType::FigUserMigrated {} => Some(
                CodewhispererterminalFigUserMigrated {
                    create_time: self.created_time,
                    value: None,
                    credential_start_url: self.credential_start_url.map(Into::into),
                }
                .into_metric_datum(),
            ),
            EventType::ChatStart { conversation_id } => Some(
                AmazonqStartChat {
                    create_time: self.created_time,
                    value: None,
                    credential_start_url: self.credential_start_url.map(Into::into),
                    amazonq_conversation_id: Some(conversation_id.into()),
                }
                .into_metric_datum(),
            ),
            EventType::ChatEnd { conversation_id } => Some(
                AmazonqEndChat {
                    create_time: self.created_time,
                    value: None,
                    credential_start_url: self.credential_start_url.map(Into::into),
                    amazonq_conversation_id: Some(conversation_id.into()),
                }
                .into_metric_datum(),
            ),
            EventType::ChatAddedMessage { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn user_logged_in() -> Event {
        Event::new(EventType::UserLoggedIn {}).await
    }

    async fn completion_inserted() -> Event {
        Event::new(EventType::CompletionInserted {
            command: "test".into(),
            terminal: Some("vscode".into()),
            shell: Some("bash".into()),
        })
        .await
    }

    async fn inline_shell_actioned() -> Event {
        Event::new(EventType::InlineShellCompletionActioned {
            session_id: "XXX".into(),
            request_id: "XXX".into(),
            suggestion_state: SuggestionState::Accept,
            edit_buffer_len: Some(123),
            suggested_chars_len: Some(42),
            latency: Duration::from_millis(500),
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

    async fn chat_start() -> Event {
        Event::new(EventType::ChatStart {
            conversation_id: "XXX".into(),
        })
        .await
    }

    async fn chat_end() -> Event {
        Event::new(EventType::ChatEnd {
            conversation_id: "XXX".into(),
        })
        .await
    }

    async fn chat_added_message() -> Event {
        Event::new(EventType::ChatAddedMessage {
            conversation_id: "XXX".into(),
            message_id: "YYY".into(),
        })
        .await
    }

    async fn all_events() -> Vec<Event> {
        vec![
            user_logged_in().await,
            completion_inserted().await,
            inline_shell_actioned().await,
            translation_actioned().await,
            cli_subcommand_executed().await,
            doctor_check_failed().await,
            dashboard_page_viewed().await,
            menu_bar_actioned().await,
            fig_user_migrated().await,
            chat_start().await,
            chat_end().await,
            chat_added_message().await,
        ]
    }

    #[tokio::test]
    async fn test_event_ser() {
        for event in all_events().await {
            let json = serde_json::to_string_pretty(&event).unwrap();
            println!("\n{json}\n");
            let deser = Event::from_json(&json).unwrap();
            assert_eq!(event, deser);
        }
    }

    #[tokio::test]
    async fn test_into_metric_datum() {
        for event in all_events().await {
            let metric_datum = event.into_metric_datum();
            if let Some(metric_datum) = metric_datum {
                println!("\n{}: {metric_datum:?}\n", metric_datum.metric_name());
            }
        }
    }
}
