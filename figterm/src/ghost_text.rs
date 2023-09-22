use std::time::{
    Duration,
    SystemTime,
};

use fig_api_client::ai::EditBufferComponent;
use fig_proto::figterm::figterm_response_message::Response as FigtermResponse;
use fig_proto::figterm::{
    FigtermResponseMessage,
    GhostTextCompleteRequest,
    GhostTextCompleteResponse,
};
use fig_request::reqwest::StatusCode;
use fig_util::directories::home_dir_utf8;
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

static LAST_RECEIVED: Lazy<Mutex<Option<SystemTime>>> = Lazy::new(|| Mutex::new(None));

static CACHE_ENABLED: Lazy<bool> = Lazy::new(|| std::env::var_os("FIG_CODEX_CACHE_DISABLE").is_none());
pub static COMPLETION_CACHE: Lazy<Mutex<radix_trie::Trie<String, f64>>> =
    Lazy::new(|| Mutex::new(radix_trie::Trie::new()));

// const DEFAULT_MIN_DURATION: Duration = Duration::from_millis(300);
// const DEFAULT_GROWTH_FACTOR: f64 = 1.5;
//
// fn growth_factor() -> f64 {
//     std::env::var("FIG_CODEX_GROWTH_FACTOR")
//         .ok()
//         .and_then(|s| s.parse().ok())
//         .unwrap_or(DEFAULT_GROWTH_FACTOR)
// }
//
// fn min_duration() -> Duration {
//     std::env::var("FIG_CODEX_DEBOUNCE_MIN_MS")
//         .ok()
//         .and_then(|s| Some(Duration::from_millis(s.parse().ok()?)))
//         .unwrap_or(DEFAULT_MIN_DURATION)
// }
//
// struct Debouncer {
//     attempt: i32,
//     min_duration: Duration,
//     max_duration: Duration,
// }
//
// impl Debouncer {
//     pub fn new(max_duration: Duration) -> Self {
//         Self {
//             attempt: 0,
//             min_duration: min_duration(),
//             max_duration,
//         }
//     }
//
//     pub fn reset(&mut self) {
//         self.attempt = 0;
//     }
//
//     pub fn delay(&mut self) -> Duration {
//         let delay = self.min_duration.mul_f64(growth_factor().powi(self.attempt));
//         self.attempt += 1;
//         delay.min(self.max_duration)
//     }
// }

pub async fn handle_request(
    figterm_request: GhostTextCompleteRequest,
    session_id: String,
    response_tx: Sender<FigtermResponseMessage>,
    history_sender: HistorySender,
) {
    if *CACHE_ENABLED {
        // use cached completion if available
        if let Some(descendant) = COMPLETION_CACHE
            .lock()
            .await
            .get_raw_descendant(&figterm_request.buffer)
        {
            let insert_text = descendant
                .iter()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(k, _)| k);

            if let Some(insert_text) = insert_text {
                let trimmed_insert = insert_text.strip_prefix(&figterm_request.buffer).unwrap_or(insert_text);

                if let Err(err) = response_tx
                    .send_async(FigtermResponseMessage {
                        response: Some(FigtermResponse::GhostTextComplete(GhostTextCompleteResponse {
                            insert_text: Some(trimmed_insert.to_owned()),
                        })),
                    })
                    .await
                {
                    error!(%err, "Failed to send ghost_text completion");
                }
                return;
            }
        }
    }

    // debounce requests
    let debounce_duration = Duration::from_millis(
        std::env::var("FIG_CODEX_DEBOUNCE_MS")
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
            warn!("Received another ghost_text completion request, aborting");
            if let Err(err) = response_tx
                .send_async(FigtermResponseMessage {
                    response: Some(FigtermResponse::GhostTextComplete(GhostTextCompleteResponse {
                        insert_text: None,
                    })),
                })
                .await
            {
                error!(%err, "Failed to send ghost_text completion");
            }

            return;
        }

        info!("Sending ghost_text completion request");

        let (history_query_tx, history_query_rx) = flume::bounded(1);
        if let Err(err) = history_sender
            .send_async(history::HistoryCommand::Query(
                HistoryQueryParams {
                    limit: std::env::var("FIG_CODEX_HISTORY_COUNT")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(25),
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

        let request = fig_api_client::ai::GhostTextRequest {
            history: history
                .into_iter()
                .map(|entry| fig_api_client::ai::CommandInfo {
                    command: entry.command,
                    cwd: entry.cwd,
                    time: entry.start_time.map(|t| t.into()),
                    exit_code: entry.exit_code,
                    hostname: entry.hostname,
                    pid: entry.pid,
                    session_id: entry.session_id,
                    shell: entry.shell,
                })
                .collect::<Vec<_>>(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            time: Some(time::OffsetDateTime::now_utc()),
            cwd: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string())),
            edit_buffer: vec![
                EditBufferComponent::String(figterm_request.buffer.clone()),
                EditBufferComponent::Other {
                    r#type: "cursor".to_string(),
                },
            ],
            home_dir: home_dir_utf8().map(|s| s.into()).ok(),
            session_id: Some(session_id.clone()),
        };

        let response = match fig_api_client::ai::request(request).await {
            Err(err) if err.is_status(StatusCode::TOO_MANY_REQUESTS) => {
                warn!("Too many requests, trying again in 1 second");
                tokio::time::sleep(Duration::from_secs(1).saturating_sub(debounce_duration)).await;
                continue;
            },
            other => other,
        };

        let insert_text = match response {
            Ok(response) => {
                let mut completion_cache = COMPLETION_CACHE.lock().await;

                for choice in &response.choices {
                    if let Some(text) = &choice.text {
                        let logprob = match &choice.logprobs {
                            Some(logprobs) => match &logprobs.token_logprobs {
                                Some(token_logprobs) => *token_logprobs.first().unwrap_or(&1.0),
                                None => 1.0,
                            },
                            None => 1.0,
                        };

                        let full_text = format!("{}{}", figterm_request.buffer, text.trim_end());

                        completion_cache.insert(full_text, logprob);
                    }
                }

                match response.choices.get(0) {
                    Some(choice) => choice.text.as_ref().map(|text| text.trim_end().to_owned()),
                    None => None,
                }
            },
            Err(err) => {
                error!(%err, "Failed to get ghost_text completion");
                None
            },
        };

        info!(?insert_text, "Got ghost_text completion");

        match response_tx
            .send_async(FigtermResponseMessage {
                response: Some(FigtermResponse::GhostTextComplete(GhostTextCompleteResponse {
                    insert_text,
                })),
            })
            .await
        {
            Ok(()) => {},
            Err(err) => {
                // This means the user typed something else before we got a response
                // We want to bump the debounce timer

                error!(%err, "Failed to send ghost_text completion");
            },
        }

        break;
    }
}
