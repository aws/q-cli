use std::borrow::Cow;
use std::cmp::Ordering as StdOrdering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};
use std::io::Write;
use std::iter::empty;
use std::process::{
    Command,
    Stdio,
};

use clap::Args;
use crossterm::style::Stylize;
use eyre::{
    bail,
    eyre,
    Result,
};
use fig_api_client::scripts::{
    scripts,
    FileType,
    Generator,
    Parameter,
    ParameterType,
    Predicate,
    Rule,
    RuleType,
    Runtime,
    Script,
    TreeElement,
};
#[cfg(unix)]
use fig_ipc::local::open_ui_element;
use fig_ipc::{
    BufferedUnixStream,
    SendMessage,
};
#[cfg(unix)]
use fig_proto::local::UiElement;
use fig_request::Request;
use fig_telemetry::{
    TrackEvent,
    TrackEventType,
    TrackSource,
};
use fig_util::directories;
use serde_json::Value as JsonValue;
#[cfg(unix)]
use skim::SkimItem;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::io::AsyncWriteExt;
use tracing::warn;
use tui::component::{
    CheckBox,
    CheckBoxEvent,
    Container,
    FilePicker,
    FilePickerEvent,
    Label,
    Paragraph,
    Select,
    SelectEvent,
    TextField,
    TextFieldEvent,
};
use tui::{
    BorderStyle,
    ColorAttribute,
    Component,
    ControlFlow,
    Event,
    EventLoop,
    InputMethod,
};
use which::which;

use crate::util::choose;

const SUPPORTED_SCHEMA_VERSION: u32 = 3;

#[derive(Debug, Args, PartialEq, Eq)]
pub struct ScriptsArgs {
    // Flags can be added here
    #[arg(allow_hyphen_values = true)]
    args: Vec<String>,
}

impl ScriptsArgs {
    pub async fn execute(self) -> Result<()> {
        execute(self.args).await
    }
}

#[cfg(unix)]
enum ScriptAction {
    Run(Box<fig_api_client::scripts::Script>),
    Create,
}

#[cfg(unix)]
impl SkimItem for ScriptAction {
    fn text(&self) -> std::borrow::Cow<str> {
        match self {
            ScriptAction::Run(script) => {
                let tags = match &script.tags {
                    Some(tags) => tags.join(" "),
                    None => String::new(),
                };

                format!(
                    "{} {} @{}/{} {}",
                    script.display_name.as_deref().unwrap_or_default(),
                    script.name,
                    script.namespace,
                    script.name,
                    tags
                )
                .into()
            },
            ScriptAction::Create => "create new script".into(),
        }
    }

    fn display<'a>(&'a self, context: skim::DisplayContext<'a>) -> skim::AnsiString<'a> {
        match self {
            ScriptAction::Run(script) => {
                let name = script.display_name.clone().unwrap_or_else(|| script.name.clone());
                let name_len = name.len();

                let tags = match &script.tags {
                    Some(tags) if !tags.is_empty() => format!(" |{}| ", tags.join("|")),
                    _ => String::new(),
                };
                let tag_len = tags.len();

                let namespace_name = format!("@{}/{}", script.namespace, script.name);
                let namespace_name_len = namespace_name.len();

                let terminal_size = crossterm::terminal::size();

                let should_constrain_width = match terminal_size {
                    Ok((term_width, _)) => term_width < 70,
                    _ => false,
                };

                if should_constrain_width {
                    skim::AnsiString::parse(&format!("{}{}", name, tags.dark_grey(),))
                } else {
                    skim::AnsiString::parse(&format!(
                        "{}{}{}{}",
                        name,
                        tags.dark_grey(),
                        " ".repeat(
                            context
                                .container_width
                                .saturating_sub(name_len)
                                .saturating_sub(tag_len)
                                .saturating_sub(namespace_name_len)
                                .saturating_sub(1)
                                .max(1)
                        ),
                        namespace_name.dark_grey()
                    ))
                }
            },
            ScriptAction::Create => skim::AnsiString::parse(&"Create new Script...".bold().blue().to_string()),
        }
    }

    fn preview(&self, _context: skim::PreviewContext) -> skim::ItemPreview {
        match self {
            ScriptAction::Run(script) => {
                let mut lines = vec![]; //format!("@{}/{}", self.namespace, self.name)];

                if let Some(description) = script.description.as_deref() {
                    if !description.is_empty() {
                        lines.push(format!("  {}", description.to_owned()));
                    } else {
                        lines.push("  No description".italic().grey().to_string())
                    }
                }

                // lines.push("━".repeat(context.width).black().to_string());
                // lines.push(self.template.clone());

                skim::ItemPreview::AnsiText(lines.join("\n"))
            },
            ScriptAction::Create => skim::ItemPreview::AnsiText("".to_string()),
        }
    }

    fn output(&self) -> std::borrow::Cow<str> {
        match self {
            ScriptAction::Run(script) => script.name.clone().into(),
            ScriptAction::Create => "".into(),
        }
    }

    fn get_matching_ranges(&self) -> Option<&[(usize, usize)]> {
        None
    }
}

async fn write_scripts() -> Result<(), eyre::Report> {
    for script in scripts(SUPPORTED_SCHEMA_VERSION).await? {
        let mut file = tokio::fs::File::create(
            directories::scripts_cache_dir()?.join(format!("{}.{}.json", script.namespace, script.name)),
        )
        .await?;
        file.write_all(serde_json::to_string_pretty(&script)?.as_bytes())
            .await?;
    }

    Ok(())
}

async fn get_scripts() -> Result<Vec<Script>> {
    let mut scripts = vec![];
    for file in directories::scripts_cache_dir()?.read_dir()?.flatten() {
        if let Some(name) = file.file_name().to_str() {
            if name.ends_with(".json") {
                let script = serde_json::from_slice::<Script>(&tokio::fs::read(file.path()).await?);

                match script {
                    Ok(script) => scripts.push(script),
                    Err(err) => eprintln!("failed to deserialize script: {}", err),
                }
            }
        }
    }

    Ok(scripts)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    String(String),
    Bool {
        val: bool,
        false_value: Option<String>,
        true_value: Option<String>,
    },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(val) => write!(f, "{}", val),
            Value::Bool {
                val,
                false_value,
                true_value,
            } => {
                if *val {
                    match true_value {
                        Some(val) => write!(f, "{val}"),
                        None => write!(f, "true"),
                    }
                } else {
                    match false_value {
                        Some(val) => write!(f, "{val}"),
                        None => write!(f, "false"),
                    }
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExecutionMethod {
    Invoke,
    Search,
}

impl std::fmt::Display for ExecutionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionMethod::Invoke => write!(f, "invoke"),
            ExecutionMethod::Search => write!(f, "search"),
        }
    }
}

pub async fn execute(env_args: Vec<String>) -> Result<()> {
    // Create cache dir
    tokio::fs::create_dir_all(directories::scripts_cache_dir()?).await?;

    let mut scripts = get_scripts().await?;

    // Must come after we get scripts
    let mut write_scripts: Option<tokio::task::JoinHandle<Result<(), _>>> = Some(tokio::spawn(write_scripts()));

    let is_interactive = atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout);

    // Parse args
    let script_name = env_args.first().map(String::from);
    let (execution_method, script) = match script_name {
        Some(name) => {
            let (namespace, name) = match name.strip_prefix('@') {
                Some(name) => match name.split('/').collect::<Vec<&str>>()[..] {
                    [namespace, name] => (Some(namespace), name),
                    _ => bail!("Malformed script specifier, expects '@namespace/script-name': {name}",),
                },
                None => (None, name.as_ref()),
            };

            let script = match namespace {
                Some(namespace) => scripts
                    .into_iter()
                    .find(|script| script.name == name && script.namespace == namespace),
                None => scripts
                    .into_iter()
                    .find(|script| script.name == name && script.is_owned_by_user),
            };

            let script = match script {
                Some(script) => script,
                None => {
                    write_scripts.take().unwrap().await??;

                    let scripts = get_scripts().await?;

                    let script = match namespace {
                        Some(namespace) => scripts
                            .into_iter()
                            .find(|script| script.name == name && script.namespace == namespace),
                        None => scripts
                            .into_iter()
                            .find(|script| script.name == name && script.is_owned_by_user),
                    };

                    match script {
                        Some(script) => script,
                        None => {
                            eprintln!("Script not found");
                            return Ok(());
                        },
                    }
                },
            };

            (ExecutionMethod::Invoke, script)
        },
        None => {
            if !is_interactive {
                bail!("No script specified");
            }

            fig_telemetry::dispatch_emit_track(
                TrackEvent::new(
                    TrackEventType::ScriptSearchViewed,
                    TrackSource::Cli,
                    env!("CARGO_PKG_VERSION").into(),
                    empty::<(&str, &str)>(),
                ),
                false,
            )
            .await
            .ok();

            if let Err(err) = write_scripts.take().unwrap().await? {
                eprintln!("Could not load remote scripts!\nFalling back to local cache.");
                warn!("Failed to acquire remote script definitions: {err}");
            }

            scripts.sort_by(|a, b| match (&a.last_invoked_at, &b.last_invoked_at) {
                (None, None) => StdOrdering::Equal,
                (None, Some(_)) => StdOrdering::Greater,
                (Some(_), None) => StdOrdering::Less,
                (Some(a), Some(b)) => match (OffsetDateTime::parse(a, &Rfc3339), OffsetDateTime::parse(b, &Rfc3339)) {
                    (Ok(a), Ok(b)) => b.cmp(&a),
                    _ => StdOrdering::Equal,
                },
            });

            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    use fig_util::desktop::{
                        launch_fig_desktop,
                        LaunchArgs,
                    };
                    use skim::prelude::*;

                    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

                    if scripts.is_empty() {
                        tx.send(Arc::new(ScriptAction::Create)).ok();
                    }

                    for script in scripts.iter() {
                        tx.send(Arc::new(ScriptAction::Run(Box::new(script.clone())))).ok();
                    }
                    drop(tx);

                    let terminal_size = crossterm::terminal::size();
                    let cursor_position = crossterm::cursor::position();

                    let height = match (terminal_size, cursor_position) {
                        (Ok((_, term_height)), Ok((_, cursor_row))) => {
                            (term_height - cursor_row).max(13).to_string()
                        }
                        _ => "100%".into()
                    };

                    let output = Skim::run_with(
                        &SkimOptionsBuilder::default()
                            .height(Some(&height))
                            .preview(Some(""))
                            .prompt(Some("▸ "))
                            .preview_window(Some("down:3"))
                            .reverse(true)
                            .case(CaseMatching::Ignore)
                            .tac(false)
                            .build()
                            .unwrap(),
                        Some(rx),
                    );

                    match output {
                        Some(out) => {
                            if out.is_abort {
                                return Ok(());
                            }

                            match out.selected_items.iter()
                                .map(|selected_item|
                                    (**selected_item)
                                        .as_any()
                                        .downcast_ref::<ScriptAction>()
                                        .unwrap()
                                        .to_owned()
                                )
                                .next() {
                                Some(script) => {
                                    match script {
                                        ScriptAction::Run(script) => (ExecutionMethod::Search, *script.clone()),
                                        ScriptAction::Create => {
                                            launch_fig_desktop(LaunchArgs {
                                                wait_for_socket: true,
                                                open_dashboard: false,
                                                immediate_update: true,
                                                verbose: true,
                                            })?;

                                            return match open_ui_element(UiElement::MissionControl, Some("/scripts".to_string())).await {
                                                Ok(()) => Ok(()),
                                                Err(err) => Err(err.into()),
                                            };
                                        },
                                    }
                                }
                                None => return Ok(()),
                            }
                        },
                        None => return Ok(()),
                    }
                } else if #[cfg(windows)] {
                    let script_names: Vec<String> = scripts
                        .iter()
                        .map(|script| {
                            script.display_name.clone().unwrap_or_else(|| script.name.clone())
                        })
                        .collect();

                    let selection = dialoguer::FuzzySelect::with_theme(&crate::util::dialoguer_theme())
                        .items(&script_names)
                        .default(0)
                        .interact()
                        .unwrap();

                    ("search", scripts.remove(selection))
                }
            }
        },
    };

    if std::env::var_os("FIG_SCRIPT_DEBUG").is_some() {
        dbg!(&script);
    }

    if execution_method == ExecutionMethod::Search {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
    }

    let script_name = format!("@{}/{}", &script.namespace, &script.name);
    if script.template_version > SUPPORTED_SCHEMA_VERSION {
        bail!(
            "Could not execute {script_name} since it requires features not available in this version of Fig.\n\
            Please update to the latest version by running {} and try again.",
            "fig update".magenta(),
        );
    }

    runtime_check(&script.runtime).await?;

    // validate that all of the script rules pass
    if let Some(ruleset) = &script.rules {
        if !rules_met(ruleset)? {
            return Ok(());
        }
    }

    let args: Vec<String> = env_args.into_iter().skip(1).collect();
    let mut arg_pairs: HashMap<String, String> = HashMap::new();

    for pair in args.chunks(2) {
        match pair[0].strip_prefix("--") {
            Some(key) => {
                arg_pairs.insert(key.to_string(), pair[1].to_string());
            },
            None => bail!("Unexpected value: {}", pair[0]),
        }
    }

    // Catch this after so we can try to catch it from the previous command parsing
    if args.len() % 2 != 0 {
        bail!("Arguments must be in the form of `--key value`");
    }

    let map = if script.parameters.is_empty() {
        Ok(HashMap::new())
    } else {
        parse_args(&arg_pairs, &script.parameters)
    };

    match (map, is_interactive) {
        (Ok(map), _) => match execution_method {
            ExecutionMethod::Invoke => {
                execute_script(script.runtime, &script.name, &script.namespace, &script.tree, &map).await?;
            },
            ExecutionMethod::Search => {
                if send_figterm(map_args_to_command(&script, &map), true).await.is_err() {
                    execute_script(script.runtime, &script.name, &script.namespace, &script.tree, &map).await?;
                }
            },
        },
        (Err(_), true) => {
            let tui_out = run_tui(&script, &arg_pairs, &script_name, &execution_method)?;

            let run_command = map_args_to_command(&script, &tui_out);

            fig_telemetry::dispatch_emit_track(
                TrackEvent::new(
                    TrackEventType::ScriptExecuted,
                    TrackSource::Cli,
                    env!("CARGO_PKG_VERSION").into(),
                    [
                        ("workflow", script_name.as_str()),
                        ("execution_method", execution_method.to_string().as_str()),
                    ],
                ),
                false,
            )
            .await
            .ok();

            if let Some(task) = write_scripts {
                if let Err(err) = task.await? {
                    eprintln!("Failed to update scripts from remote: {err}");
                }
            }

            if send_figterm(run_command, true).await.is_err() {
                execute_script(script.runtime, &script.name, &script.namespace, &script.tree, &tui_out).await?;
            }
        },
        (Err(err), false) => {
            eprintln!("{}", err);
            std::process::exit(1);
        },
    }

    Ok(())
}

fn parse_args(args: &HashMap<String, String>, parameters: &Vec<Parameter>) -> Result<HashMap<String, Value>> {
    let mut missing_args: Vec<String> = Vec::new();
    let mut map = HashMap::new();
    for parameter in parameters {
        let value = args.get(&parameter.name);
        map.insert(parameter.name.clone(), match value {
            Some(value) => match &parameter.parameter_type {
                ParameterType::Checkbox {
                    true_value_substitution,
                    false_value_substitution,
                } => match value.as_str() {
                    "true" => Value::Bool {
                        val: true,
                        true_value: Some(true_value_substitution.clone()),
                        false_value: Some(false_value_substitution.clone()),
                    },
                    "false" => Value::Bool {
                        val: false,
                        true_value: Some(true_value_substitution.clone()),
                        false_value: Some(false_value_substitution.clone()),
                    },
                    _ => bail!(
                        "Invalid value for checkbox {}, must be `true` or `false`",
                        parameter.name
                    ),
                },
                _ => Value::String(value.clone()),
            },
            None => {
                missing_args.push(parameter.name.clone());
                continue;
            },
        });
    }

    if !missing_args.is_empty() {
        bail!("Missing required arguments: {}", missing_args.join(", "));
    }

    Ok(map)
}

fn map_args_to_command(script: &Script, args: &HashMap<String, Value>) -> String {
    let mut command = format!("fig run {}", match script.is_owned_by_user {
        true => script.name.clone(),
        false => format!("@{}/{}", &script.namespace, &script.name),
    });
    for (arg, val) in args {
        use std::fmt::Write;

        match val {
            Value::String(s) => write!(command, " --{arg} {}", escape(s.into())).ok(),
            Value::Bool { val, .. } => write!(command, " --{arg} {val}").ok(),
        };
    }

    command
}

async fn send_figterm(text: String, execute: bool) -> eyre::Result<()> {
    let session_id = std::env::var("FIGTERM_SESSION_ID")?;
    let mut conn = BufferedUnixStream::connect(fig_util::directories::figterm_socket_path(&session_id)?).await?;
    conn.send_message(fig_proto::figterm::FigtermRequestMessage {
        request: Some(fig_proto::figterm::figterm_request_message::Request::InsertOnNewCmd(
            fig_proto::figterm::InsertOnNewCmdRequest {
                text: format!("\x1b[200~{text}\x1b[201~"),
                execute,
            },
        )),
    })
    .await?;
    Ok(())
}

async fn execute_script(
    runtime: Runtime,
    name: &str,
    namespace: &str,
    tree: &[TreeElement],
    args: &HashMap<String, Value>,
) -> Result<()> {
    let start_time = time::OffsetDateTime::now_utc();

    let script = tree.iter().fold(String::new(), |mut acc, branch| {
        match branch {
            TreeElement::String(string) => acc.push_str(string.as_str()),
            TreeElement::Token { name } => acc.push_str(&match &args[name.as_str()] {
                Value::String(string) => string.clone(),
                Value::Bool {
                    val,
                    true_value,
                    false_value,
                } => match val {
                    true => true_value.clone().unwrap_or_else(|| "true".into()),
                    false => false_value.clone().unwrap_or_else(|| "false".into()),
                },
            }),
        }
        acc
    });

    // determine that runtime exists before validating rules
    let (mut command, text) = match runtime {
        Runtime::Bash => {
            let mut command = Command::new("bash");
            command.arg("-c");
            command.arg(script);
            (command, None)
        },
        Runtime::Python => {
            let mut command = Command::new("python3");
            command.arg("-c");
            command.arg(script);
            (command, None)
        },
        Runtime::Node => {
            let mut command = Command::new("node");
            command.arg("-e");
            command.arg(script);
            (command, None)
        },
        Runtime::Deno => {
            let mut command = Command::new("deno");
            command.arg("run");
            command.arg("-A");
            command.arg("-");
            command.stdin(Stdio::piped());

            (command, Some(script))
        },
    };

    let mut child = command.spawn()?;
    if let Some(text) = text {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(text.as_bytes())?;
        stdin.flush()?;
    }

    let exit_code = child.wait().ok().and_then(|output| output.code());
    if let Ok(execution_start_time) = start_time.format(&Rfc3339) {
        if let Ok(execution_duration) = i64::try_from((OffsetDateTime::now_utc() - start_time).whole_nanoseconds()) {
            Request::post(format!("/workflows/{name}/invocations"))
                .body(serde_json::json!({
                    "namespace": namespace,
                    "commandStderr": JsonValue::Null,
                    "exitCode": exit_code,
                    "executionStartTime": execution_start_time,
                    "executionDuration": execution_duration,
                }))
                .auth()
                .send()
                .await
                .ok();
        }
    }

    Ok(())
}

fn non_whitelisted(ch: char) -> bool {
    !matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '=' | '/' | ',' | '.' | '+')
}

/// Escape characters that may have special meaning in a shell, including spaces.
pub fn escape(s: Cow<str>) -> Cow<str> {
    if !s.is_empty() && !s.contains(non_whitelisted) {
        return s;
    }

    let mut es = String::with_capacity(s.len() + 2);
    es.push('\'');
    for ch in s.chars() {
        match ch {
            '\'' | '!' => {
                es.push_str("'\\");
                es.push(ch);
                es.push('\'');
            },
            _ => es.push(ch),
        }
    }
    es.push('\'');
    es.into()
}

async fn runtime_check(runtime: &Runtime) -> Result<()> {
    match which(runtime.exe()) {
        Ok(_) => Ok(()),
        Err(_) => try_install(runtime),
    }
}

fn try_install(runtime: &Runtime) -> Result<()> {
    let confirm = |name: &str| {
        matches!(
            choose(
                &format!("{runtime:?} is not installed. Would you like to install it with {name}?"),
                &["Yes", "No"],
            ),
            Ok(0)
        )
    };

    // if not interactive, don't try to install
    if !atty::is(atty::Stream::Stdout) {
        eyre::bail!("Failed to execute script, {runtime:?} is not installed");
    }

    // If brew is installed, use it to install the dependency
    if which("brew").is_ok() && confirm("brew") {
        let mut command = Command::new("brew");
        command.arg("install");
        command.arg("--quiet");
        command.arg(runtime.brew_package());

        command.env("HOMEBREW_NO_AUTO_UPDATE", "1");
        command.env("HOMEBREW_NO_ENV_HINTS", "1");

        command.status()?;
        return Ok(());
    }

    eyre::bail!("Failed to execute script, {runtime:?} is not installed");
}

fn rules_met(ruleset: &Vec<Vec<Rule>>) -> Result<bool> {
    for set in ruleset {
        let mut set_met = set.is_empty();
        for rule in set {
            let query = match rule.key {
                RuleType::WorkingDirectory => std::env::current_dir()?.to_string_lossy().to_string(),
                RuleType::GitRemote => String::from_utf8(
                    Command::new("git")
                        .args(["remote", "get-url", "origin"])
                        .output()?
                        .stdout,
                )?,
                RuleType::ContentsOfDirectory => {
                    std::env::current_dir()?
                        .read_dir()?
                        .fold(String::new(), |acc, path| match path {
                            Ok(path) => format!("{acc}{}\n", path.file_name().to_string_lossy()),
                            Err(_) => acc,
                        })
                },
                RuleType::GitRootDirectory => String::from_utf8(
                    Command::new("git")
                        .args(["rev-parse", "--show-toplevel"])
                        .output()?
                        .stdout,
                )?,
                RuleType::EnvironmentVariable => todo!(),
                RuleType::CurrentBranch => String::from_utf8(
                    Command::new("git")
                        .args(["rev-parse", "--abbrev-ref", "HEAD"])
                        .output()?
                        .stdout,
                )?,
            };

            let query = query.trim();

            let mut rule_met = match rule.predicate {
                Predicate::Contains => query.contains(&rule.value),
                Predicate::Equals => query == rule.value,
                Predicate::Matches => regex::Regex::new(&rule.value)?.is_match(query),
                Predicate::StartsWith => query.starts_with(&rule.value),
                Predicate::EndsWith => query.ends_with(&rule.value),
                Predicate::Exists => !query.is_empty(),
            };

            if rule.inverted {
                rule_met = !rule_met;
            }

            set_met |= rule_met;
        }

        if !set_met {
            eprintln!(
                "{}",
                match set.len() == 1 {
                    true => "The following rule must be met:",
                    false => "One of the following rules must be met:",
                }
                .red()
            );

            for rule in set {
                eprintln!("- {rule}");
            }

            eprintln!();

            return Ok(false);
        }
    }

    Ok(true)
}

fn run_tui(
    script: &Script,
    arg_pairs: &HashMap<String, String>,
    script_name: &str,
    execution_method: &ExecutionMethod,
) -> Result<HashMap<String, Value>> {
    let style_sheet = tui::style_sheet! {
        "*" => {
            border_left_color: ColorAttribute::PaletteIndex(8);
            border_right_color: ColorAttribute::PaletteIndex(8);
            border_top_color: ColorAttribute::PaletteIndex(8);
            border_bottom_color: ColorAttribute::PaletteIndex(8);
            border_style: BorderStyle::Ascii {
                top_left: '┌',
                top: '─',
                top_right: '┐',
                left: '│',
                right: '│',
                bottom_left: '└',
                bottom: '─',
                bottom_right: '┘',
            };
        },
        "*:focus" => {
            color: ColorAttribute::PaletteIndex(3);
            border_left_color: ColorAttribute::PaletteIndex(3);
            border_right_color: ColorAttribute::PaletteIndex(3);
            border_top_color: ColorAttribute::PaletteIndex(3);
            border_bottom_color: ColorAttribute::PaletteIndex(3);
            border_style: BorderStyle::Ascii {
                top_left: '┏',
                top: '━',
                top_right: '┓',
                left: '┃',
                right: '┃',
                bottom_left: '┗',
                bottom: '━',
                bottom_right: '┛',
            };
        },
        "input:checkbox" => {
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "div" => {
            color: ColorAttribute::PaletteIndex(8);
            width: Some(100.0);
            padding_top: -1.0;
            border_left_width: 1.0;
            border_top_width: 1.0;
            border_bottom_width: 1.0;
            border_right_width: 1.0;
        },
        "h1" => {
            margin_left: 1.0;
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "p" => {
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "select" => {
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "input:text" => {
            width: Some(98.0);
            padding_left: 1.0;
            padding_right: 2.0;
        },
        "#__header" => {
            margin_bottom: 1.0;
        },
        "#__keybindings" => {
            margin_left: 0.0;
            width: Some(110.0);
        },
        "#__view" => {
            border_style: BorderStyle::None;
            padding_top: 0.0;
            margin_left: 2.0;
            margin_right: 2.0;
        },
        "#__form" => {
            padding_top: 0.0;
            border_style: BorderStyle::None;
        },
        "#__description" => {
            margin_top: 1000.0;
            height: Some(1.0);
        }
    };

    let mut header = Paragraph::new("__header")
        .push_line_break()
        .push_styled_text(script.display_name.as_ref().unwrap_or(&script.name), None, None, true)
        .push_styled_text(
            format!(" • @{}", script.namespace),
            Some(ColorAttribute::PaletteIndex(8)),
            None,
            false,
        );

    if let Some(description) = &script.description {
        if !description.is_empty() {
            header = header.push_line_break().push_styled_text(
                description,
                Some(ColorAttribute::PaletteIndex(3)),
                None,
                false,
            );
        }
    }

    let mut form = Container::new("__form");

    let mut args: HashMap<String, Value> = HashMap::new();
    let mut description_map = HashMap::new();
    let mut flag_map: HashMap<String, (String, String)> = HashMap::new();

    for parameter in &script.parameters {
        if let Some(description) = &parameter.description {
            description_map.insert(parameter.name.to_owned(), description.to_owned());
        }

        let mut property = Container::new("").push(Label::new(
            &parameter.name,
            parameter.display_name.as_ref().unwrap_or(&parameter.name),
            false,
        ));

        let parameter_value = arg_pairs.get(&parameter.name).cloned().unwrap_or_default();
        match &parameter.parameter_type {
            ParameterType::Selector {
                placeholder,
                suggestions,
                generators,
            } => {
                let mut options = suggestions.to_owned().unwrap_or_default();
                if let Some(generators) = generators {
                    for generator in generators {
                        match generator {
                            Generator::Named { .. } => {
                                return Err(eyre!("named generators aren't supported in scripts yet"));
                            },
                            Generator::Script { script } => {
                                if let Ok(output) = Command::new("bash").arg("-c").arg(script).output() {
                                    for suggestion in String::from_utf8_lossy(&output.stdout).split('\n') {
                                        if !suggestion.is_empty() {
                                            options.push(suggestion.to_owned());
                                        }
                                    }
                                }
                            },
                        }
                    }
                }

                property = property.push(
                    Select::new(&parameter.name, options, true)
                        .with_text(parameter_value)
                        .with_hint(placeholder.as_deref().unwrap_or("Search...")),
                );
            },
            ParameterType::Text { placeholder } => {
                property = property.push(
                    TextField::new(&parameter.name)
                        .with_text(parameter_value.to_string())
                        .with_hint(placeholder.to_owned().unwrap_or_default()),
                )
            },
            ParameterType::Checkbox {
                true_value_substitution,
                false_value_substitution,
            } => {
                flag_map.insert(
                    parameter.name.to_owned(),
                    (true_value_substitution.to_owned(), false_value_substitution.to_owned()),
                );

                let checked = arg_pairs
                    .get(&parameter.name)
                    .map(|val| match val.as_str() {
                        "true" => true,
                        val if val == true_value_substitution => true,
                        _ => false,
                    })
                    .unwrap_or(false);

                args.insert(parameter.name.to_owned(), Value::Bool {
                    val: checked,
                    true_value: Some(true_value_substitution.to_owned()),
                    false_value: Some(false_value_substitution.to_owned()),
                });

                property = property.push(CheckBox::new(
                    &parameter.name,
                    parameter.description.to_owned().unwrap_or_else(|| "Toggle".to_string()),
                    checked,
                ));
            },
            ParameterType::Path { file_type, extensions } => {
                let (files, folders) = match file_type {
                    FileType::Any => (true, true),
                    FileType::FileOnly => (true, false),
                    FileType::FolderOnly => (false, true),
                };

                property = property.push(FilePicker::new(
                    &parameter.name,
                    std::env::current_dir()?,
                    files,
                    folders,
                    extensions.clone(),
                ));
            },
        };

        form = form.push(property);
    }

    let mut view = Container::new("__view")
        .push(header)
        .push(form)
        .push(Paragraph::new("__description"))
        .push(
            Paragraph::new("__keybindings")
                .push_styled_text("enter", Some(ColorAttribute::PaletteIndex(3)), None, false)
                .push_styled_text(" select • ", Some(ColorAttribute::Default), None, false)
                .push_styled_text("tab", Some(ColorAttribute::PaletteIndex(3)), None, false)
                .push_styled_text(" next • ", Some(ColorAttribute::Default), None, false)
                .push_styled_text("shift+tab", Some(ColorAttribute::PaletteIndex(3)), None, false)
                .push_styled_text(" previous • ", Some(ColorAttribute::Default), None, false)
                .push_styled_text("⎵", Some(ColorAttribute::PaletteIndex(3)), None, false)
                .push_styled_text(" toggle • ", Some(ColorAttribute::Default), None, false)
                .push_styled_text("⌃o", Some(ColorAttribute::PaletteIndex(3)), None, false)
                .push_styled_text(" preview", Some(ColorAttribute::Default), None, false),
        );

    let mut temp = None;

    EventLoop::new().run(
        &mut view,
        InputMethod::Form,
        style_sheet,
        |event, view, control_flow| match event {
            Event::Quit => *control_flow = ControlFlow::Quit,
            Event::Terminate => {
                let handle = tokio::runtime::Handle::current();
                let script_name = script_name.to_owned();
                let execution_method = execution_method.to_owned();
                std::thread::spawn(move || {
                    handle
                        .block_on(fig_telemetry::dispatch_emit_track(
                            TrackEvent::new(
                                TrackEventType::ScriptCancelled,
                                TrackSource::Cli,
                                env!("CARGO_PKG_VERSION").into(),
                                [
                                    ("workflow", script_name),
                                    ("execution_method", execution_method.to_string()),
                                ],
                            ),
                            false,
                        ))
                        .ok();
                })
                .join()
                .ok();

                *control_flow = ControlFlow::Quit;
            },
            Event::TempChangeView => {
                if let Some(temp) = temp.take() {
                    view.remove("__preview");
                    view.insert("__header", temp);
                    return;
                }

                let colors = [
                    ColorAttribute::PaletteIndex(13),
                    ColorAttribute::PaletteIndex(12),
                    ColorAttribute::PaletteIndex(14),
                ];

                let mut paragraph = Paragraph::new("");
                for element in &script.tree {
                    match element {
                        TreeElement::String(s) => paragraph = paragraph.push_text(s),
                        TreeElement::Token { name } => {
                            let mut hasher = DefaultHasher::new();
                            name.hash(&mut hasher);
                            let hash = hasher.finish() as usize;

                            paragraph = paragraph.push_styled_text(
                                match args.get(name.as_str()) {
                                    Some(value) => value.to_string(),
                                    None => format!("{{{{{name}}}}}"),
                                },
                                Some(colors[hash % colors.len()]),
                                None,
                                false,
                            );
                        },
                    }
                }

                temp = view.remove("__form");
                view.insert(
                    "__header",
                    Box::new(
                        Container::new("__preview")
                            .push(Label::new("preview_label", "Preview", false))
                            .push(paragraph),
                    ),
                );
            },
            Event::FocusChanged { id, focus } => {
                if focus {
                    let description = description_map.get(&id).cloned().unwrap_or_default();
                    view.replace(
                        "__description",
                        Box::new(Paragraph::new("__description").push_styled_text(
                            description,
                            Some(ColorAttribute::PaletteIndex(8)),
                            None,
                            false,
                        )),
                    );
                }
            },
            Event::CheckBox(event) => match event {
                CheckBoxEvent::Checked { id, checked } => {
                    let (true_val, false_val) = flag_map.get(&id).unwrap();

                    args.insert(id, Value::Bool {
                        val: checked,
                        false_value: Some(false_val.to_owned()),
                        true_value: Some(true_val.to_owned()),
                    });
                },
            },
            Event::FilePicker(event) => match event {
                FilePickerEvent::FilePathChanged { id, path } => {
                    args.insert(id, Value::String(path.to_string_lossy().to_string()));
                },
            },
            Event::Select(event) => match event {
                SelectEvent::OptionSelected { id, option } => {
                    args.insert(id, Value::String(option));
                },
            },
            Event::TextField(event) => match event {
                TextFieldEvent::TextChanged { id, text } => {
                    args.insert(id, Value::String(text));
                },
            },
            _ => (),
        },
    )?;

    Ok(args)
}

#[cfg(test)]
mod test {
    use fig_api_client::scripts::Script;

    use super::*;

    #[test]
    fn test_rules() -> Result<()> {
        let json = serde_json::json!(
            {
                "name": "eekum-bokum",
                "displayName": "Eekum Bokum",
                "description": "Quick snippet for git push",
                "templateVersion": 0,
                "tags": [
                    "git"
                ],
                "rules": [
                    [
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "EQUALS"
                        },
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "EQUALS"
                        }
                    ],
                    [
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "CONTAINS"
                        },
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "CONTAINS"
                        }
                    ],
                    [
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "STARTSWITH"
                        },
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "STARTSWITH"
                        }
                    ],
                    [
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "ENDSWITH"
                        },
                        {
                            "key": "Working-Directory",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "ENDSWITH"
                        }
                    ],
                    [
                        {
                            "key": "Working-Directory",
                            "value": ".",
                            "inverted": false,
                            "predicate": "MATCHES"
                        },
                        {
                            "key": "Working-Directory",
                            "value": ".",
                            "inverted": true,
                            "predicate": "MATCHES"
                        }
                    ],
                    [
                        {
                            "key": "Current-Branch",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "CONTAINS"
                        },
                        {
                            "key": "Current-Branch",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "CONTAINS"
                        }
                    ],
                    [
                        {
                            "key": "Contents-Of-Directory",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "CONTAINS"
                        },
                        {
                            "key": "Contents-Of-Directory",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "CONTAINS"
                        }
                    ],
                    [
                        {
                            "key": "Git-Remote",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "CONTAINS"
                        },
                        {
                            "key": "Git-Remote",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "CONTAINS"
                        }
                    ],
                    [
                        {
                            "key": "Git-Root-Directory",
                            "value": "package.json",
                            "inverted": false,
                            "predicate": "CONTAINS"
                        },
                        {
                            "key": "Git-Root-Directory",
                            "value": "package.json",
                            "inverted": true,
                            "predicate": "CONTAINS"
                        }
                    ]
                ],
                "namespace": "chay-at-fig",
                "parameters": [],
                "template": "echo \"hello :)\"",
                "tree": [
                    "echo \"hello :)\""
                ],
                "isOwnedByUser": true
            }
        );

        let script = serde_json::from_value::<Script>(json)?;
        assert!(script.rules.is_some());

        let ruleset = script.rules.unwrap();
        rules_met(&ruleset)?;

        Ok(())
    }
}
