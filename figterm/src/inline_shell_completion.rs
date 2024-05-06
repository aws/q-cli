use std::fmt::Write;
use std::time::{
    Duration,
    SystemTime,
};

use fig_api_client::ai::{
    CodeWhispererFileContext,
    LanguageName,
    ProgrammingLanguage,
};
use fig_proto::figterm::figterm_response_message::Response as FigtermResponse;
use fig_proto::figterm::{
    FigtermResponseMessage,
    InlineShellCompletionRequest,
    InlineShellCompletionResponse,
};
use fig_settings::history::CommandInfo;
use flume::Sender;
use once_cell::sync::Lazy;
use radix_trie::TrieCommon;
use tokio::sync::Mutex;
use tracing::{
    error,
    info,
    warn,
};

use crate::history::{
    self,
    HistoryQueryParams,
    HistorySender,
};

static LAST_RECEIVED: Mutex<Option<SystemTime>> = Mutex::const_new(None);

static CACHE_ENABLED: Lazy<bool> = Lazy::new(|| std::env::var_os("Q_INLINE_SHELL_COMPLETION_CACHE_DISABLE").is_none());
pub static COMPLETION_CACHE: Lazy<Mutex<radix_trie::Trie<String, f64>>> =
    Lazy::new(|| Mutex::new(radix_trie::Trie::new()));

pub async fn handle_request(
    figterm_request: InlineShellCompletionRequest,
    _session_id: String,
    response_tx: Sender<FigtermResponseMessage>,
    history_sender: HistorySender,
) {
    let buffer = figterm_request.buffer.trim_start();

    if *CACHE_ENABLED {
        // use cached completion if available
        if let Some(descendant) = COMPLETION_CACHE.lock().await.get_raw_descendant(buffer) {
            let insert_text = descendant
                .iter()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(k, _)| k);

            if let Some(insert_text) = insert_text {
                let trimmed_insert = insert_text.strip_prefix(buffer).unwrap_or(insert_text);

                if let Err(err) = response_tx
                    .send_async(FigtermResponseMessage {
                        response: Some(FigtermResponse::InlineShellCompletion(InlineShellCompletionResponse {
                            insert_text: Some(trimmed_insert.to_owned()),
                        })),
                    })
                    .await
                {
                    error!(%err, "Failed to send inline_shell_completion completion");
                }
                return;
            }
        }
    }

    // debounce requests
    let debounce_duration = Duration::from_millis(
        std::env::var("Q_INLINE_SHELL_COMPLETION_DEBOUNCE_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(300),
    );

    let now = SystemTime::now();
    LAST_RECEIVED.lock().await.replace(now);

    for _ in 0..3 {
        tokio::time::sleep(debounce_duration).await;
        if *LAST_RECEIVED.lock().await == Some(now) {
            // TODO: determine behavior here, None or Some(unix timestamp)
            *LAST_RECEIVED.lock().await = Some(SystemTime::now());
        } else {
            warn!("Received another inline_shell_completion completion request, aborting");
            if let Err(err) = response_tx
                .send_async(FigtermResponseMessage {
                    response: Some(FigtermResponse::InlineShellCompletion(InlineShellCompletionResponse {
                        insert_text: None,
                    })),
                })
                .await
            {
                error!(%err, "Failed to send inline_shell_completion completion");
            }

            return;
        }

        info!("Sending inline_shell_completion completion request");

        let (history_query_tx, history_query_rx) = flume::bounded(1);
        if let Err(err) = history_sender
            .send_async(history::HistoryCommand::Query(
                HistoryQueryParams {
                    limit: std::env::var("Q_INLINE_SHELL_COMPLETION_HISTORY_COUNT")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(50),
                },
                history_query_tx,
            ))
            .await
        {
            error!(%err, "Failed to send history query");
        }

        let history = match history_query_rx.recv_async().await {
            Ok(Some(history)) => history,
            err => {
                error!(?err, "Failed to get history");
                vec![]
            },
        };

        let prompt = prompt(&history, buffer);

        let request = fig_api_client::ai::CodeWhispererRequest {
            file_context: CodeWhispererFileContext {
                left_file_content: prompt,
                right_file_content: "".into(),
                filename: "history.sh".into(),
                programming_language: ProgrammingLanguage {
                    language_name: LanguageName::Shell,
                },
            },
            max_results: 1,
            next_token: None,
        };

        let response = match fig_api_client::ai::request_cw(request)
            .await
            .map_err(|err| err.into_service_error())
        {
            Err(err) if err.is_throttling_error() => {
                warn!(%err, "Too many requests, trying again in 1 second");
                tokio::time::sleep(Duration::from_secs(1).saturating_sub(debounce_duration)).await;
                continue;
            },
            other => other,
        };

        let insert_text = match response {
            Ok(response) => {
                let recommendations = response.completions.unwrap_or_default();
                let mut completion_cache = COMPLETION_CACHE.lock().await;

                let mut recommendations = recommendations
                    .into_iter()
                    .map(|choice| {
                        choice
                            .content
                            .split_once('\n')
                            .map_or(&*choice.content, |(l, _)| l)
                            .trim_end()
                            .to_owned()
                    })
                    .collect::<Vec<_>>();

                for recommendation in &recommendations {
                    let full_text = format!("{buffer}{recommendation}");
                    completion_cache.insert(full_text, 1.0);
                }

                recommendations.first_mut().map(std::mem::take)
            },
            Err(err) => {
                error!(%err, "Failed to get inline_shell_completion completion");
                None
            },
        };

        info!(?insert_text, "Got inline_shell_completion completion");

        match response_tx
            .send_async(FigtermResponseMessage {
                response: Some(FigtermResponse::InlineShellCompletion(InlineShellCompletionResponse {
                    insert_text,
                })),
            })
            .await
        {
            Ok(()) => {},
            Err(err) => {
                // This means the user typed something else before we got a response
                // We want to bump the debounce timer

                error!(%err, "Failed to send inline_shell_completion completion");
            },
        }

        break;
    }
}

fn prompt(history: &[CommandInfo], buffer: &str) -> String {
    history
        .iter()
        .rev()
        .filter_map(|c| c.command.clone())
        .chain([buffer.into()])
        .enumerate()
        .fold(String::new(), |mut acc, (i, c)| {
            if i > 0 {
                acc.push('\n');
            }
            let _ = write!(acc, "{:>5}  {c}", i + 1);
            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt() {
        let history = vec![
            CommandInfo {
                command: Some("echo world".into()),
                ..Default::default()
            },
            CommandInfo {
                command: Some("echo hello".into()),
                ..Default::default()
            },
        ];

        let prompt = prompt(&history, "echo ");
        println!("{prompt}");

        assert_eq!(prompt, "    1  echo hello\n    2  echo world\n    3  echo ");
    }
}
