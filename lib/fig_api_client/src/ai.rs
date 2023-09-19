use fig_request::Method;
use url::Url;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandInfo {
    pub command: Option<String>,
    pub shell: Option<String>,
    pub pid: Option<i32>,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub time: Option<time::OffsetDateTime>,
    pub hostname: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum EditBufferComponent {
    String(String),
    Other { r#type: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexRequest {
    pub history: Vec<CommandInfo>,
    pub os: String,
    pub arch: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub time: Option<time::OffsetDateTime>,
    pub edit_buffer: Vec<EditBufferComponent>,
    pub cwd: Option<String>,
    pub home_dir: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodexChoice {
    pub text: Option<String>,
    pub index: Option<i32>,
    pub finish_reason: Option<String>,
    pub logprobs: Option<LogProbs>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogProbs {
    pub token_logprobs: Option<Vec<f64>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexResponse {
    pub choices: Vec<CodexChoice>,
}

pub async fn request(request: CodexRequest) -> fig_request::Result<CodexResponse> {
    match std::env::var("FIG_CODEX_API_URL") {
        Ok(url) => fig_request::Request::new_with_url(Method::POST, Url::parse(&url).unwrap()),
        Err(_) => fig_request::Request::post("/ai/codex"),
    }
    .auth()
    .body_json(&request)
    .deser_json()
    .await
}
