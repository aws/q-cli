use std::sync::{
    Arc,
    Mutex,
};
use std::time::Duration;

use amzn_codewhisperer_client::config::interceptors::BeforeTransmitInterceptorContextMut;
use amzn_codewhisperer_client::config::{
    Intercept,
    RuntimeComponents,
    StalledStreamProtectionConfig,
};
use amzn_codewhisperer_client::error::{
    DisplayErrorContext,
    SdkError,
};
use amzn_codewhisperer_client::operation::generate_completions::{
    GenerateCompletionsError,
    GenerateCompletionsOutput,
};
use amzn_codewhisperer_client::operation::list_available_customizations::ListAvailableCustomizationsError;
use amzn_codewhisperer_client::types::error::AccessDeniedError;
use amzn_codewhisperer_client::types::AccessDeniedExceptionReason;
use auth::builder_id::BearerResolver;
use aws_config::{
    BehaviorVersion,
    SdkConfig,
};
use aws_credential_types::Credentials;
use aws_smithy_runtime_api::http::Response;
use aws_smithy_types::config_bag::ConfigBag;
use fig_aws_common::{
    app_name,
    UserAgentOverrideInterceptor,
};
use serde_json::Value;
use tracing::error;

use crate::customization::Customization;
pub use crate::endpoints::Endpoint;

// Opt out constants
const SHARE_CODEWHISPERER_CONTENT_SETTINGS_KEY: &str = "codeWhisperer.shareCodeWhispererContentWithAWS";
const X_AMZN_CODEWHISPERER_OPT_OUT_HEADER: &str = "x-amzn-codewhisperer-optout";

// Session ID constants
const X_AMZN_SESSIONID_HEADER: &str = "x-amzn-sessionid";

fn is_codewhisperer_content_optout() -> bool {
    !fig_settings::settings::get_bool_or(SHARE_CODEWHISPERER_CONTENT_SETTINGS_KEY, true)
}

#[derive(Debug, Clone)]
struct OptOutInterceptor;

impl Intercept for OptOutInterceptor {
    fn name(&self) -> &'static str {
        "OptOutInterceptor"
    }

    fn modify_before_signing(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), amzn_codewhisperer_client::error::BoxError> {
        context.request_mut().headers_mut().insert(
            X_AMZN_CODEWHISPERER_OPT_OUT_HEADER,
            if is_codewhisperer_content_optout() {
                "true"
            } else {
                "false"
            },
        );

        Ok(())
    }
}

async fn sdk_config(endpoint: &Endpoint) -> SdkConfig {
    aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(endpoint.region())
        .credentials_provider(Credentials::new("xxx", "xxx", None, None, "xxx"))
        .load()
        .await
}

pub async fn cw_client(endpoint: Endpoint) -> amzn_codewhisperer_client::Client {
    let conf_builder: amzn_codewhisperer_client::config::Builder = (&sdk_config(&endpoint).await).into();
    let conf = conf_builder
        .interceptor(OptOutInterceptor)
        .interceptor(UserAgentOverrideInterceptor::new())
        .bearer_token_resolver(BearerResolver)
        .app_name(app_name())
        .endpoint_url(endpoint.url())
        .build();
    amzn_codewhisperer_client::Client::from_conf(conf)
}

pub async fn cw_streaming_client(endpoint: Endpoint) -> amzn_codewhisperer_streaming_client::Client {
    let conf_builder: amzn_codewhisperer_streaming_client::config::Builder = (&sdk_config(&endpoint).await).into();
    // @ptrucks recommends using the same configuration as here
    // https://code.amazon.com/packages/MynahChatTests/blobs/b3e23c24e5abce0150f872883f80efc0df50ebbc/--/src/main.rs#L37-L41
    let stalled_stream_protection_config = StalledStreamProtectionConfig::enabled()
        .grace_period(Duration::from_secs(20))
        .build();
    let conf = conf_builder
        .interceptor(OptOutInterceptor)
        .interceptor(UserAgentOverrideInterceptor::new())
        .bearer_token_resolver(BearerResolver)
        .app_name(app_name())
        .endpoint_url(endpoint.url())
        .stalled_stream_protection(stalled_stream_protection_config)
        .build();
    amzn_codewhisperer_streaming_client::Client::from_conf(conf)
}

pub fn cw_endpoint() -> Endpoint {
    match fig_settings::state::get_value("api.codewhisperer.service") {
        Ok(Some(Value::Object(o))) => {
            let endpoint = o.get("endpoint").and_then(|v| v.as_str());
            let region = o.get("region").and_then(|v| v.as_str());

            match (endpoint, region) {
                (Some(endpoint), Some(region)) => Endpoint::Custom {
                    url: endpoint.to_owned().into(),
                    region: region.to_owned().into(),
                },
                _ => Endpoint::Prod,
            }
        },
        _ => Endpoint::Prod,
    }
}

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
pub struct AiRequest {
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

pub struct CwResponse {
    pub output: GenerateCompletionsOutput,
    pub session_id: Option<String>,
}

/// Make a request to the CodeWhisperer API with no special error handling
async fn cw_generate_completions_raw(
    client: &amzn_codewhisperer_client::Client,
    request: CodeWhispererRequest,
    session_id_lock: Arc<Mutex<Option<String>>>,
) -> Result<
    GenerateCompletionsOutput,
    SdkError<GenerateCompletionsError, aws_smithy_runtime_api::client::orchestrator::HttpResponse>,
> {
    #[derive(Debug, Clone)]
    struct SessionIdInterceptor(Arc<Mutex<Option<String>>>);

    impl Intercept for SessionIdInterceptor {
        fn name(&self) -> &'static str {
            "SessionIdInterceptor"
        }

        fn read_after_deserialization(
            &self,
            context: &amzn_codewhisperer_client::config::interceptors::AfterDeserializationInterceptorContextRef<'_>,
            _runtime_components: &RuntimeComponents,
            _cfg: &mut ConfigBag,
        ) -> Result<(), amzn_codewhisperer_client::error::BoxError> {
            *self.0.lock().expect("Failed to write to SessionIdInterceptor mutex") = context
                .response()
                .headers()
                .get(X_AMZN_SESSIONID_HEADER)
                .map(Into::into);
            Ok(())
        }
    }

    let customization_arn = match Customization::load_selected() {
        Ok(opt) => opt.map(|Customization { arn, .. }| arn),
        Err(err) => {
            error!(%err, "Failed to load selected customization");
            None
        },
    };

    client
        .generate_completions()
        .file_context(
            amzn_codewhisperer_client::types::FileContext::builder()
                .left_file_content(request.file_context.left_file_content)
                .right_file_content(request.file_context.right_file_content)
                .filename(request.file_context.filename)
                .programming_language(
                    amzn_codewhisperer_client::types::ProgrammingLanguage::builder()
                        .language_name(request.file_context.programming_language.language_name.as_ref())
                        .build()?,
                )
                .build()?,
        )
        .max_results(request.max_results)
        .set_next_token(request.next_token)
        .set_customization_arn(customization_arn)
        .customize()
        .interceptor(SessionIdInterceptor(session_id_lock.clone()))
        .send()
        .await
}

/// Make a request to the CodeWhisperer API with error handling for invalid customizations with the
/// specified client
async fn cw_generate_completions(
    client: amzn_codewhisperer_client::Client,
    request: CodeWhispererRequest,
) -> Result<CwResponse, SdkError<GenerateCompletionsError, aws_smithy_runtime_api::client::orchestrator::HttpResponse>>
{
    let session_id_lock = Arc::new(Mutex::new(None));
    let res = cw_generate_completions_raw(&client, request.clone(), session_id_lock.clone()).await;

    let output = match res {
        Ok(output) => output,
        Err(ref err @ SdkError::ServiceError(ref service_err))
            if matches!(
                service_err.err(),
                GenerateCompletionsError::AccessDeniedError(AccessDeniedError {
                    reason: Some(AccessDeniedExceptionReason::UnauthorizedCustomizationResourceAccess),
                    ..
                })
            ) =>
        {
            error!(err =% DisplayErrorContext(err), "Access denied for selected customization, clearing selection and trying again");
            if let Err(err) = Customization::delete_selected() {
                error!(%err, "Failed to delete selected customization");
            }
            cw_generate_completions_raw(&client, request, session_id_lock.clone()).await?
        },
        Err(err) => return Err(err),
    };

    let session_id = session_id_lock
        .lock()
        .expect("Failed to read from SessionIdInterceptor mutex")
        .clone();
    Ok(CwResponse { output, session_id })
}

/// Make a request to the CodeWhisperer API with the default client
pub async fn request_cw(
    request: CodeWhispererRequest,
) -> Result<CwResponse, SdkError<GenerateCompletionsError, aws_smithy_runtime_api::client::orchestrator::HttpResponse>>
{
    cw_generate_completions(cw_client(cw_endpoint()).await, request).await
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeWhispererFileContext {
    pub left_file_content: String,
    pub right_file_content: String,
    pub filename: String,
    pub programming_language: ProgrammingLanguage,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgrammingLanguage {
    pub language_name: LanguageName,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, strum::AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LanguageName {
    Python,
    Javascript,
    Java,
    Csharp,
    Typescript,
    C,
    Cpp,
    Go,
    Kotlin,
    Php,
    Ruby,
    Rust,
    Scala,
    Shell,
    Sql,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeWhispererReferenceTrackerConfiguration {
    pub recommendations_with_references: RecommendationsWithReferences,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecommendationsWithReferences {
    Block,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeWhispererRequest {
    pub file_context: CodeWhispererFileContext,
    pub max_results: i32,
    // pub reference_tracker_configuration: CodeWhispererReferenceTrackerConfiguration,
    pub next_token: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodewhispererResponse {
    pub recommendations: Vec<CodewhispererRecommendation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodewhispererRecommendation {
    pub content: Option<String>,
}

/// List the customizations the user has access to
pub async fn list_customizations() -> Result<Vec<Customization>, SdkError<ListAvailableCustomizationsError, Response>> {
    let mut customizations = Vec::new();
    let mut paginator = cw_client(cw_endpoint())
        .await
        .list_available_customizations()
        .into_paginator()
        .send();
    while let Some(res) = paginator.next().await {
        let output = res?;
        customizations.extend(output.customizations.into_iter().map(Into::into));
    }
    Ok(customizations)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command;

    use super::*;

    #[tokio::test]
    async fn client_tests() {
        let endpoint = Endpoint::Prod;
        cw_client(endpoint.clone()).await;
        cw_streaming_client(endpoint).await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_request() {
        tracing_subscriber::fmt().init();

        // check for $HOME/.toolbox/bin in path

        // Client::from_conf(Config::builder().(&sdk).endpoint());

        let history = "1516  11.10.2023 14:20  cd Documents
1517  11.10.2023 14:20  gh repo clone neovim/neovim
1518  11.10.2023 14:20  cd neovim
1519  11.10.2023 14:20  git log --reverse
1520  11.10.2023 14:32  git status
1521  11.10.2023 14:32  git add .
1523  11.10.2023 14:32  git status
1524  11.10.2023 14:32  git diff --staged
1525  11.10.2023 14:32  git status
1526  11.10.2023 14:33  git commit -m 'Update build script' -n
1527  11.10.2023 14:33  git push
1528  11.10.2023 14:33  git stash
1529  11.10.2023 14:33  git pull -r
1530  11.10.2023 14:33  git stash pop
1531  11.10.2023 14:33  git push
1532  11.10.2023 14:28  brazil ws sync
1533  11.10.2023 14:39  git status
1534  11.10.2023 14:39  git add .
1535  11.10.2023 14:39  git commit -m 'Set beta account number' -n
1536  11.10.2023 14:39  cr
1537  11.10.2023 15:00  git add .
1538  11.10.2023 15:01  brazil ws sync
1539  11.10.2023 15:10  cd lib/shell-color
1540  11.10.2023 15:10  cargo rt
1541  11.10.2023 15:10  cargo t
1542  11.10.2023 15:18  git status
1543  11.10.2023 15:18  git diff --staged
1544  11.10.2023 15:18  brazil ws sync
1545  11.10.2023 15:18  git stash
1546  11.10.2023 15:18  brazil ws sync
1547  11.10.2023 15:19  git stash pop
1548  11.10.2023 15:19  git add .
1549  11.10.2023 15:19  git commit -m 'Add gamma stage' -n
1550  11.10.2023 15:19  git push
1551  11.10.2023 15:19  cr
1552  11.10.2023 15:20  mwi";

        cw_client(cw_endpoint()).await;

        let mut request = CodeWhispererRequest {
            file_context: CodeWhispererFileContext {
                left_file_content: history.into(),
                right_file_content: "".into(),
                filename: "history.sh".into(),
                programming_language: ProgrammingLanguage {
                    language_name: LanguageName::Shell,
                },
            },
            max_results: 1,
            next_token: None,
        };

        let time = std::time::Instant::now();
        let mut res = request_cw(request.clone()).await.unwrap().output;

        println!("{res:?}");
        println!("time: {:?}", time.elapsed());
        for (i, a) in res.completions.unwrap_or_default().iter().enumerate() {
            println!("rec {i}: {:?}", a.content);
        }

        let time = std::time::Instant::now();

        while let Some(token) = &res.next_token {
            println!("next_token: {token:?}");
            if token.is_empty() {
                break;
            } else {
                request.next_token = Some(token.clone());
                res = request_cw(request.clone()).await.unwrap().output;
                println!("{res:?}");
                println!("time: {:?}", time.elapsed());
                for (i, a) in res.completions.unwrap_or_default().iter().enumerate() {
                    println!("rec {i}: {:?}", a.content);
                }
            }
        }

        // let res2 = request_cw(CodeWhispererRequest {
        //     file_context: CodeWhispererFileContext {
        //         left_file_content: "# List the files in the directory that have a p in
        // them\n".into(),         right_file_content: "".into(),
        //         filename: "history.sh".into(),
        //         programming_language: ProgrammingLanguage {
        //             language_name: LanguageName::Shell,
        //         },
        //     },
        //     max_results: 1,
        //     next_token: None,
        // })
        // .await
        // .unwrap();

        // println!("time: {:?}", time.elapsed());
        // for (i, a) in res2.recommendations.unwrap_or_default().iter().enumerate() {
        //     println!("rec {i}: {:?}", a.content)
        // }

        // left_file_content: Some("".into()),
        // right_file_content: None,
        // filename: None,
        // programming_language: Some(ProgrammingLanguage {
        //     language_name: Some("shell".into()),
        // }),

        // let res = codewhisperer_raw_request(CodeWhispererRequest {
        //     file_context: CodeWhispererFileContext {
        //         left_file_content: "# List the files in the directory\n".into(),
        //         right_file_content: "".into(),
        //         filename: "history.sh".into(),
        //         programming_language: ProgrammingLanguage {
        //             language_name: LanguageName::Shell,
        //         },
        //     },
        //     max_results: 1,
        //     next_token: "".into(),
        // })
        // .await;
    }

    #[tokio::test]
    #[ignore]
    async fn claude_test() {
        // #!/bin/sh
        //
        // MODEL=claude-v2
        //
        // for i in "$@"; do
        // case $i in
        // -m=*|--model=*)
        // MODEL="${i#*=}"
        // shift # past argument=value
        // ;;
        // -*|--*)
        // echo "Unknown option $i"
        // exit 1
        // ;;
        // )
        // ;;
        // esac
        // done
        //
        // PROMPT="{\"prompt\":\"Human: $@\\n\\nAssistant: \"}"
        // IFS=$'\n'
        // mcurl -s -N "https://llm-playground.pdebie.people.a2z.com/api/model/$MODEL/stream" \
        // -H 'content-type: application/json' \
        // --data-raw "$PROMPT" |
        // sed -u 's/data: {"text":"//; s/"}//; s/\\n/NEWLINE/g; s/\\r//g; /data: {"closing"/d' |
        // while read i; do
        // /bin/echo -n "$i" | sed -u 's/NEWLINE/\n/g'
        // done
        // echo

        let question = "How do I write a bash script?";

        let model = "bedrock-claude-instant-v1";
        // let prompt = r#"{"prompt":"Human: {}\n\nAssistant: "}"#;
        let prompt = serde_json::json!({
            "prompt": format!("Human: {}\n\nAssistant: ", question),
        })
        .to_string();
        let url = format!(
            "https://llm-playground.pdebie.people.a2z.com/api/model/{}/stream",
            model
        );

        let mut command = Command::new("mcurl");
        command
            .args([
                "-s",
                "-N",
                &url,
                "-H",
                "content-type: application/json",
                "--data-raw",
                &prompt,
            ])
            .stdout(std::process::Stdio::piped());
        let child = command.spawn().unwrap();
        let stdout = child.stdout.unwrap();
        let buffer = tokio::io::BufReader::new(stdout);

        let mut lines = buffer.lines();

        while let Some(line) = lines.next_line().await.unwrap() {
            match line.strip_prefix("data: ") {
                Some(data) => {
                    let data: Value = serde_json::from_str(data).unwrap();
                    if let Some(text) = data.get("text").and_then(|d| d.as_str()) {
                        print!("{text}");
                    }
                },
                None => {
                    // println!("Unknown: {:?}", line);
                },
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_customizations() {
        let a = list_customizations().await.unwrap();
        for c in a {
            println!("{c:?}");
            if c.name.as_deref() == Some("Amazon-Internal-V17") {
                c.save_selected().unwrap();
            }
        }
    }
}
