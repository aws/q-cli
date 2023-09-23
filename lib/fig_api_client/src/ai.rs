use std::process::Command;

use amzn_codewhisperer_client::operation::generate_recommendations::{
    GenerateRecommendationsError,
    GenerateRecommendationsOutput,
};
use amzn_codewhisperer_client::Client;
use aws_credential_types::provider::ProvideCredentials;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::SdkError;
use fig_request::Method;
use fig_util::directories::home_dir;
use http::response::Response;
use once_cell::sync::Lazy;
use serde_json::Value;
use tokio::sync::OnceCell;
use tracing::info;
use url::Url;

const DEFAULT_REGION: &str = "us-east-1";
// "https://rts.alpha-us-west-2.codewhisperer.ai.aws.dev"
const CODEWHISPERER_ENDPOINT: &str = "https://codewhisperer.us-east-1.amazonaws.com";
const APP_NAME: &str = "figTest";

static INJECT_TOOLBOX_BIN: Lazy<()> = Lazy::new(|| {
    let toolbox_bin = fig_util::directories::home_dir().unwrap().join(".toolbox/bin");
    if toolbox_bin.exists() {
        let mut paths = std::env::split_paths(&std::env::var_os("PATH").unwrap()).collect::<Vec<_>>();
        if !paths.contains(&toolbox_bin) {
            paths.insert(0, toolbox_bin);
        }
        std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    }
});

fn aws_profile() -> Option<String> {
    fig_settings::state::get_string("aws.profile").ok().flatten()
}

static AWS_CLIENT: OnceCell<Client> = OnceCell::const_new();

async fn cw_client() -> &'static Client {
    AWS_CLIENT
        .get_or_init(|| async {
            *INJECT_TOOLBOX_BIN;

            let sdk = aws_config::from_env()
                .region(DEFAULT_REGION)
                .profile_name(aws_profile().unwrap_or_else(|| "DEFAULT".into()))
                .load()
                .await;

            let conf_builder: amzn_codewhisperer_client::config::Builder = (&sdk).into();
            let conf = conf_builder
                .app_name(aws_config::AppName::new(APP_NAME).unwrap())
                .endpoint_url(CODEWHISPERER_ENDPOINT)
                .build();

            Client::from_conf(conf)
        })
        .await
}

pub fn init() {
    tokio::spawn(async {
        cw_client().await;
    });
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

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct CodexChoice {
//     pub text: Option<String>,
// }

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct CodexResponse {
//     pub choices: Vec<CodexChoice>,
// }

// pub async fn request(request: AiRequest) -> fig_request::Result<CodexResponse> {
//     let codewhisperer_request = CodewhipererRequest {
//         file_context: CodewhipererFileContext {
//             left_file_content: match &request.edit_buffer[0] {
//                 EditBufferComponent::String(s) => {
//                     let history: Vec<_> = request
//                         .history
//                         .iter()
//                         .rev()
//                         // .take(20)
//                         .filter_map(|c| c.command.clone())
//                         .collect();

//                     let prompt = format!("{}\n{}", history.join("\n"), s);
//                     prompt
//                 },
//                 EditBufferComponent::Other { r#type: _ } => return Ok(CodexResponse { choices:
// vec![] }),             }
//             .into(),
//             right_file_content: "".into(),
//             filename: "history.sh".into(),
//             programming_language: ProgrammingLanguage {
//                 language_name: LanguageName::Shell,
//             },
//         },
//         max_results: 1,
//         next_token: "".into(),
//     };

//     let res = request_cw(codewhisperer_request).await.unwrap();

//     info!(?res, "Codewhisperer response");

//     let text = match res.recommendations.first() {
//         Some(r) => r.content.clone(),
//         None => return Ok(CodexResponse { choices: vec![] }),
//     };

//     Ok(CodexResponse {
//         choices: vec![CodexChoice { text: Some(text) }],
//     })
// }

pub async fn request_cw(
    CodewhipererRequest {
        file_context:
            CodewhipererFileContext {
                left_file_content,
                right_file_content,
                filename,
                programming_language,
            },
        max_results,
        next_token,
    }: CodewhipererRequest,
) -> Result<GenerateRecommendationsOutput, SdkError<GenerateRecommendationsError, Response<SdkBody>>> {
    cw_client()
        .await
        .generate_recommendations()
        .file_context(
            amzn_codewhisperer_client::types::FileContext::builder()
                .left_file_content(left_file_content)
                .right_file_content(right_file_content)
                .filename(filename)
                .programming_language(
                    amzn_codewhisperer_client::types::ProgrammingLanguage::builder()
                        .language_name(programming_language.language_name.as_ref())
                        .build(),
                )
                .build(),
        )
        .max_results(max_results)
        .next_token(next_token.unwrap_or_default())
        .send()
        .await
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodewhipererFileContext {
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
pub struct CodewhipererReferenceTrackerConfiguration {
    pub recommendations_with_references: RecommendationsWithReferences,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecommendationsWithReferences {
    Block,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodewhipererRequest {
    pub file_context: CodewhipererFileContext,
    pub max_results: i32,
    // pub reference_tracker_configuration: CodewhipererReferenceTrackerConfiguration,
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

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use tokio::io::AsyncBufReadExt;
    use tokio::process::Command;

    use super::*;

    // #[tokio::test]
    // async fn test_request() {
    //     // let config = aws_config::from_env()
    //     //     .profile_name("personal")
    //     //     .region("us-east-1")
    //     //     .load()
    //     //     .await;

    //     // dbg!(config.credentials_provider().unwrap().provide_credentials().await);

    //     let codewhisperer_request = CodewhipererRequest {
    //         file_context: CodewhipererFileContext {
    //             left_file_content: "# List the files in the directory\n".into(),
    //             right_file_content: "".into(),
    //             filename: "history.sh".into(),
    //             programming_language: ProgrammingLanguage {
    //                 language_name: LanguageName::Shell,
    //             },
    //         },
    //         max_results: 3,
    //         reference_tracker_configuration: CodewhipererReferenceTrackerConfiguration {
    //             recommendations_with_references: RecommendationsWithReferences::BLOCK,
    //         },
    //     };

    //     let json = serde_json::to_string(&codewhisperer_request).unwrap();

    //     let client = aws_smithy_client::Client::builder()
    //         .dyn_https_connector(Default::default())
    //         .middleware(tower::layer::util::Identity::default())
    //         .build();

    //     let request = http::Request::builder()
    //         .uri("https://codewhisperer.us-east-1.amazonaws.com")
    //         .body(SdkBody::from(json))
    //         .unwrap();

    //     println!("{:?}", request);

    //     #[derive(Debug, Clone)]
    //     struct Error;
    //     impl ProvideErrorKind for Error {
    //         fn retryable_error_kind(&self) -> Option<aws_smithy_types::retry::ErrorKind> {
    //             todo!()
    //         }

    //         fn code(&self) -> Option<&str> {
    //             todo!()
    //         }
    //     }

    //     impl std::error::Error for Error {}

    //     impl std::fmt::Display for Error {
    //         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //             todo!()
    //         }
    //     }

    //     #[derive(Debug, Clone)]
    //     struct TestParseResponse;
    //     impl ParseStrictResponse for TestParseResponse {
    //         type Output = Result<String, Error>;

    //         fn parse(&self, response: &http::Response<Bytes>) -> Self::Output {
    //             Ok(String::from_utf8(response.body().to_vec()).unwrap())
    //         }
    //     }

    //     let op = Operation::new(Request::new(request), TestParseResponse);
    //     let res = client.call(op).await.unwrap();

    //     println!("{:#?}", res);
    // }

    #[tokio::test]
    async fn test_request() {
        tracing_subscriber::fmt().init();

        // check for $HOME/.toolbox/bin in path

        // Client::from_conf(Config::builder().(&sdk).endpoint());

        cw_client().await;

        let time = std::time::Instant::now();
        let res = request_cw(CodewhipererRequest {
            file_context: CodewhipererFileContext {
                left_file_content: "# List the files in the directory\n".into(),
                right_file_content: "".into(),
                filename: "history.sh".into(),
                programming_language: ProgrammingLanguage {
                    language_name: LanguageName::Shell,
                },
            },
            max_results: 1,
            next_token: None,
        })
        .await
        .unwrap();

        println!("time: {:?}", time.elapsed());
        for (i, a) in res.recommendations.unwrap_or_default().iter().enumerate() {
            println!("rec {i}: {:?}", a.content)
        }

        let time = std::time::Instant::now();

        let res2 = request_cw(CodewhipererRequest {
            file_context: CodewhipererFileContext {
                left_file_content: "# List the files in the directory that have a p in them\n".into(),
                right_file_content: "".into(),
                filename: "history.sh".into(),
                programming_language: ProgrammingLanguage {
                    language_name: LanguageName::Shell,
                },
            },
            max_results: 1,
            next_token: None,
        })
        .await
        .unwrap();

        println!("time: {:?}", time.elapsed());
        for (i, a) in res2.recommendations.unwrap_or_default().iter().enumerate() {
            println!("rec {i}: {:?}", a.content)
        }

        // left_file_content: Some("".into()),
        // right_file_content: None,
        // filename: None,
        // programming_language: Some(ProgrammingLanguage {
        //     language_name: Some("shell".into()),
        // }),

        // let res = codewhisperer_raw_request(CodewhipererRequest {
        //     file_context: CodewhipererFileContext {
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
                    let data: Value = serde_json::from_str(&data).unwrap();
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
}
