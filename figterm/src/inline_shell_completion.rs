use std::fmt::Write;
use std::path::Path;
use std::time::{
    Duration,
    Instant,
    SystemTime,
};

use aws_types::request_id::RequestId;
use fig_api_client::ai::{
    CodeWhispererFileContext,
    CwResponse,
    LanguageName,
    ProgrammingLanguage,
};
use fig_proto::figterm::figterm_response_message::Response as FigtermResponse;
use fig_proto::figterm::{
    FigtermResponseMessage,
    InlineShellCompletionAcceptRequest,
    InlineShellCompletionRequest,
    InlineShellCompletionResponse,
};
use fig_settings::history::CommandInfo;
use fig_telemetry::{
    Event,
    SuggestionState,
};
use fig_util::terminal::{
    CURRENT_TERMINAL,
    CURRENT_TERMINAL_VERSION,
};
use fig_util::Shell;
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
static COMPLETION_CACHE: Lazy<Mutex<radix_trie::Trie<String, f64>>> = Lazy::new(|| Mutex::new(radix_trie::Trie::new()));

static TELEMETRY_QUEUE: Lazy<Mutex<TelemetryQueue>> = Lazy::new(|| Mutex::new(TelemetryQueue::new()));

pub async fn on_prompt() {
    *COMPLETION_CACHE.lock().await = radix_trie::Trie::new();
    TELEMETRY_QUEUE.lock().await.send_all_items().await;
}

struct TelemetryQueue {
    items: Vec<TelemetryQueueItem>,
}

impl TelemetryQueue {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    async fn send_all_items(&mut self) {
        let start_url = auth::builder_id_token().await.ok().flatten().and_then(|t| t.start_url);
        for item in self.items.drain(..) {
            let TelemetryQueueItem {
                timestamp,
                session_id,
                request_id,
                suggestion_state,
                edit_buffer_len,
                suggested_chars_len,
                latency,
                ..
            } = item;

            fig_telemetry::send_event(Event {
                created_time: Some(timestamp),
                credential_start_url: start_url.clone(),
                ty: fig_telemetry::EventType::InlineShellCompletionActioned {
                    session_id,
                    request_id,
                    suggestion_state,
                    edit_buffer_len,
                    suggested_chars_len,
                    latency,
                    terminal: CURRENT_TERMINAL.as_ref().map(|s| s.internal_id().into_owned()),
                    terminal_version: CURRENT_TERMINAL_VERSION.as_ref().map(|s| s.clone()),
                    // The only supported shell currently is Zsh
                    shell: Some(Shell::Zsh.as_str().into()),
                    shell_version: None,
                },
            })
            .await;
        }
    }
}

struct TelemetryQueueItem {
    buffer: String,
    suggestion: String,

    timestamp: SystemTime,

    session_id: String,
    request_id: String,
    suggestion_state: SuggestionState,
    edit_buffer_len: Option<i64>,
    suggested_chars_len: Option<i64>,
    latency: Duration,
}

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

        let start_instant = Instant::now();

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
            Ok(CwResponse { output, session_id }) => {
                let request_id = output.request_id().unwrap().to_owned();
                let completions = output.completions.unwrap_or_default();
                let mut completion_cache = COMPLETION_CACHE.lock().await;

                let mut completions = completions
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

                for recommendation in &completions {
                    let full_text = format!("{buffer}{recommendation}");
                    completion_cache.insert(full_text, 1.0);
                }

                if let Some(suggestion) = completions.first_mut() {
                    let full_text = format!("{buffer}{suggestion}");
                    let buffer = buffer.to_owned();

                    let mut suggestion_state = if suggestion.is_empty() {
                        SuggestionState::Empty
                    } else {
                        SuggestionState::Accept
                    };

                    let valid = validate(&full_text);
                    if !valid {
                        suggestion_state = SuggestionState::Discard;
                    }

                    let suggestion_clone = suggestion.clone();
                    tokio::spawn(async move {
                        TELEMETRY_QUEUE.lock().await.items.push(TelemetryQueueItem {
                            suggested_chars_len: suggestion_clone.chars().count().try_into().ok(),
                            suggestion: suggestion_clone,
                            timestamp: SystemTime::now(),
                            session_id: session_id.unwrap_or_default(),
                            request_id,
                            latency: start_instant.elapsed(),
                            suggestion_state,
                            edit_buffer_len: buffer.chars().count().try_into().ok(),
                            buffer,
                        });
                    });

                    if valid { Some(std::mem::take(suggestion)) } else { None }
                } else {
                    None
                }
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

pub async fn handle_accept(figterm_request: InlineShellCompletionAcceptRequest, _session_id: String) {
    let mut queue = TELEMETRY_QUEUE.lock().await;
    for item in queue.items.iter_mut() {
        if item.buffer == figterm_request.buffer.trim_start() && item.suggestion == figterm_request.suggestion {
            item.suggestion_state = SuggestionState::Accept;
        }
    }
    queue.send_all_items().await;
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

fn validate(command: &str) -> bool {
    validate_with_context(
        command,
        || {
            std::env::current_dir()
                .ok()
                .and_then(|s| s.to_str().map(ToOwned::to_owned))
        },
        || fig_util::directories::home_dir_utf8().ok(),
        |key| std::env::var(key).map(Some),
    )
}

fn validate_with_context<CwdStr, Cwd, HdStr, Hd, CtxStr, Ctx, E>(
    command: &str,
    cwd: Cwd,
    home_dir: Hd,
    ctx: Ctx,
) -> bool
where
    CwdStr: AsRef<str>,
    Cwd: FnOnce() -> Option<CwdStr>,
    HdStr: AsRef<str>,
    Hd: FnOnce() -> Option<HdStr>,
    CtxStr: AsRef<str>,
    Ctx: FnMut(&str) -> Result<Option<CtxStr>, E>,
{
    let command = command.trim();

    // Currently the api responds with redactions of `XXX` if there is PII identified, just filter
    // these to improve quality
    if command.contains("XXX") {
        return false;
    }

    // Try to validate the args to "cd",
    if let Some(args) = shlex::split(command) {
        if args.first().map(|s| s.as_str()) == Some("cd") && args.len() == 2 {
            if let Some(arg) = args.get(1) {
                if let Ok(arg) = shellexpand::full_with_context(arg, home_dir, ctx) {
                    let path = Path::new(arg.as_ref());
                    if path.is_absolute() && !path.is_dir() {
                        return false;
                    }
                    let canonicalized = cwd().and_then(|cwd| Path::new(cwd.as_ref()).join(path).canonicalize().ok());
                    match canonicalized {
                        Some(p) if !p.is_dir() => return false,
                        Some(_) => {},
                        None => return false,
                    }
                }
            }
        }
    }

    true
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

    #[test]
    fn test_validate() {
        let tempdir = tempfile::tempdir().unwrap();

        // Create structure to test with
        std::fs::create_dir_all(tempdir.path().join("a")).unwrap();
        std::fs::create_dir_all(tempdir.path().join("a").join("b")).unwrap();
        std::fs::create_dir_all(tempdir.path().join("space in name")).unwrap();

        std::fs::write(tempdir.path().join("file"), "hello").unwrap();
        std::fs::write(tempdir.path().join("a").join("file"), "hello").unwrap();

        let tempdir_path = tempdir.path().to_str().unwrap();
        let cwd = || -> Option<&str> { Some(tempdir_path) };
        let home_dir = || -> Option<&str> { Some(tempdir_path) };
        let context = |key: &str| -> anyhow::Result<Option<&str>> {
            match key {
                "HOME" => anyhow::Ok(Some(tempdir_path)),
                "A" => anyhow::Ok(Some("a")),
                "B" => anyhow::Ok(Some("b")),
                _ => Ok(None),
            }
        };
        let valid = |response: &str| validate_with_context(response, cwd, home_dir, context);

        // Allows normal commands
        assert!(valid(r#"echo "hello""#));
        assert!(valid(r#"aws s3 cp file s3://bucket"#));

        // Rejects redactions
        assert!(!valid(r#"git clone "XXXXXXXXXXXXXXXXXXXXXXXXXX""#));
        assert!(!valid(r#"echo "XXX""#));
        assert!(!valid(r#"curl -X POST "XXXXXXXXXXXXXXXXXXXXXXXXXX""#));

        // Allows cd commands with folders that eixts
        assert!(valid(r#"cd /tmp"#));
        assert!(valid(r#"cd "/tmp""#));
        assert!(valid(r#"cd ~"#));
        assert!(valid(r#"cd ~/"#));
        assert!(valid(r#"cd ~/a"#));
        assert!(valid(r#"cd ~/a/"#));
        assert!(valid(r#"cd ~/a/b"#));
        assert!(valid(r#"cd ~/$A"#));
        assert!(valid(r#"cd ~/$A/$B"#));
        assert!(valid(r#"cd ~/space\ in\ name"#));
        assert!(valid(r#"cd ~/"space in name""#));
        assert!(valid(r#"cd a"#));
        assert!(valid(r#"cd a/"#));
        assert!(valid(r#"cd a/b"#));
        assert!(valid(r#"cd $A"#));
        assert!(valid(r#"cd $A/$B"#));
        assert!(valid(r#"cd space\ in\ name"#));
        assert!(valid(r#"cd "space in name""#));

        // Rejects cd commands that don't exist or are not folders
        assert!(!valid(r#"cd /folder/doesnt/exist"#));
        assert!(!valid(r#"cd /file"#));
        assert!(!valid(r#"cd ~/file"#));
        assert!(!valid(r#"cd ~/a/file"#));
        assert!(!valid(r#"cd ~/$A/file"#));
        assert!(!valid(r#"cd ~/$A/$B/file"#));
        assert!(!valid(r#"cd b"#));
        assert!(!valid(r#"cd b/"#));
        assert!(!valid(r#"cd file"#));
        assert!(!valid(r#"cd $B"#));
        assert!(!valid(r#"cd a/b/file"#));

        // Cases that are not currently rejected due to ambiguity
        assert!(valid(r#"cd space in name"#));
    }
}
