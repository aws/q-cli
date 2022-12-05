use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};
use std::io::Write;
use std::iter::empty;
use std::path::Path;
use std::process::{
    Command,
    Stdio,
};

use bytes::BytesMut;
use clap::Args;
use crossterm::style::Stylize;
use eyre::{
    bail,
    eyre,
    Result,
};
use fig_api_client::scripts::{
    sync_scripts,
    FileType,
    Generator,
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
use fig_util::consts::FIG_SCRIPTS_SCHEMA_VERSION;
use fig_util::directories;
#[cfg(unix)]
use skim::SkimItem;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::io::{
    AsyncReadExt,
    AsyncWriteExt,
};
use tui::component::{
    CheckBox,
    CheckBoxEvent,
    Container,
    FilePicker,
    FilePickerEvent,
    Layout,
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

async fn get_scripts() -> Result<Vec<Script>> {
    let scripts_cache_dir = directories::scripts_cache_dir()?;
    tokio::fs::create_dir_all(&scripts_cache_dir).await?;

    if scripts_cache_dir.read_dir()?.count() == 0 {
        sync_scripts().await?;
    }

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

pub async fn execute(command_arguments: Vec<String>) -> Result<()> {
    let mut scripts = get_scripts().await?;
    let mut join_write_scripts = Some(tokio::spawn(sync_scripts()));

    let is_interactive = atty::is(atty::Stream::Stdin)
        && atty::is(atty::Stream::Stdout)
        && std::env::var_os("FIG_SCRIPT_EXECUTION").is_none();

    // Parse args
    let script_name = command_arguments.first().map(String::from);
    let (execution_method, mut script) = match script_name {
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
                    join_write_scripts.take().unwrap().await??;
                    scripts = get_scripts().await?;

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
                false,
            )
            .await
            .ok();

            // 1. All scripts user have personally ever invoked, ordered recency
            // 2. All scripts other people on team have ever invoked, ordered by their recency
            // 3. All other scripts in alphabetical order
            scripts.sort_by(|a, b| match (a.last_invoked_at_by_user, b.last_invoked_at_by_user) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (None, None) => match (a.last_invoked_at, b.last_invoked_at) {
                    (Some(a), Some(b)) => a.cmp(&b),
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (None, None) => a.name.cmp(&b.name),
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

                    for script in scripts.iter().rev() {
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

                    (ExecutionMethod::Search, scripts.remove(selection))
                }
            }
        },
    };

    if join_write_scripts
        .as_ref()
        .map(|join| join.is_finished())
        .unwrap_or_default()
    {
        // This is always okay to unwrap because we just checked that it's finished
        if let Ok(Ok(scripts)) = join_write_scripts.take().unwrap().await {
            // Find the script again in case it was updated
            match scripts
                .into_iter()
                .find(|new_script| new_script.namespace == script.namespace && new_script.name == script.name)
            {
                Some(new_script) => script = new_script,
                None => {
                    eprintln!("Script is no longer available");
                    return Ok(());
                },
            }
        }
    }

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
    if script.template_version > FIG_SCRIPTS_SCHEMA_VERSION {
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

    match command_arguments.len() {
        0 | 1 if is_interactive && !script.parameters.is_empty() => {
            let values_by_arg = run_tui(&script, &HashMap::new(), &script_name, &execution_method)?;

            let mut missing_args = vec![];
            for parameter in &script.parameters {
                if !values_by_arg.contains_key(&parameter.name) {
                    missing_args.push(parameter.name.to_owned());
                }
            }

            if !missing_args.is_empty() {
                bail!("Missing required arguments: {}", missing_args.join(", "));
            }

            let telem_join = tokio::spawn(fig_telemetry::dispatch_emit_track(
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
                true,
            ));

            execute_script_or_insert(&script, &values_by_arg).await?;

            if let Some(write_scripts) = join_write_scripts.take() {
                write_scripts.await?.ok();
            }

            telem_join.await.ok();
        },
        _ => {
            let mut command = clap::Command::new(&script_name);

            if let Some(description) = &script.description {
                command = command.about(description);
            }

            for param in &script.parameters {
                let mut arg = clap::Arg::new(&param.name).long(&param.name);

                if param.name.len() == 1 {
                    arg = arg.short(param.name.chars().next().unwrap());
                }

                if let Some(description) = &param.description {
                    arg = arg.help(description);
                }

                match param.parameter_type {
                    ParameterType::Selector { .. } => {
                        command = command.arg(arg.value_parser(clap::value_parser!(String)).required(true));
                    },
                    ParameterType::Text { .. } | ParameterType::Path { .. } => {
                        command = command.arg(arg.value_parser(clap::value_parser!(String)).required(true));
                    },
                    ParameterType::Checkbox { .. } => {
                        command = command.arg(arg.action(clap::ArgAction::SetTrue));
                    },
                };
            }

            let mut matches = command.get_matches_from(command_arguments);
            let mut map: HashMap<String, Value> = HashMap::new();

            for param in &script.parameters {
                match &param.parameter_type {
                    ParameterType::Selector { .. } | ParameterType::Text { .. } | ParameterType::Path { .. } => {
                        if let Some(value) = matches.remove_one::<String>(&param.name) {
                            map.insert(param.name.clone(), Value::String(value));
                        }
                    },
                    ParameterType::Checkbox {
                        false_value_substitution,
                        true_value_substitution,
                    } => {
                        map.insert(param.name.clone(), Value::Bool {
                            val: matches.get_flag(&param.name),
                            false_value: Some(false_value_substitution.clone()),
                            true_value: Some(true_value_substitution.clone()),
                        });
                    },
                };
            }

            match execution_method {
                ExecutionMethod::Invoke => {
                    execute_script(&script, &map).await?;
                },
                ExecutionMethod::Search => {
                    execute_script_or_insert(&script, &map).await?;
                },
            }
        },
    };

    if let Some(write_scripts) = join_write_scripts.take() {
        write_scripts.await?.ok();
    }

    Ok(())
}

fn map_args_to_command(script: &Script, args: &HashMap<String, Value>) -> String {
    let mut command = format!("fig run {}", match script.is_owned_by_user {
        true => script.name.clone(),
        false => format!("@{}/{}", &script.namespace, &script.name),
    });
    for (arg, val) in args {
        use std::fmt::Write;

        match val {
            Value::String(s) => {
                write!(command, " --{arg} {}", escape(s.into())).ok();
            },
            Value::Bool { val: true, .. } => {
                write!(command, " --{arg}").ok();
            },
            Value::Bool { val: false, .. } => {},
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

async fn execute_script(script: &Script, args: &HashMap<String, Value>) -> Result<()> {
    let start_time = time::OffsetDateTime::now_utc();

    let templated_script = script.tree.iter().fold(String::new(), |mut acc, branch| {
        match branch {
            TreeElement::String(string) => acc.push_str(string.as_str()),
            TreeElement::Token { name } => acc.push_str(&match &args[name.as_str()] {
                Value::String(string) => match script.runtime {
                    Runtime::Bash => string.clone(),
                    Runtime::Python | Runtime::Node | Runtime::Deno => {
                        serde_json::to_string(string).expect("Failed to serialize string to JSON string")
                    },
                },
                Value::Bool {
                    val,
                    true_value,
                    false_value,
                } => match (&script.runtime, val) {
                    (Runtime::Bash, true) => true_value.clone().unwrap_or_else(|| "true".into()),
                    (Runtime::Bash, false) => false_value.clone().unwrap_or_else(|| "false".into()),
                    (Runtime::Python, true) => "True".into(),
                    (Runtime::Python, false) => "False".into(),
                    (Runtime::Node | Runtime::Deno, true) => "true".into(),
                    (Runtime::Node | Runtime::Deno, false) => "false".into(),
                },
            }),
        }
        acc
    });

    // determine that runtime exists before validating rules
    let (mut command, text) = match script.runtime {
        Runtime::Bash => {
            let mut command = tokio::process::Command::new("bash");
            command.arg("-c");
            command.arg(templated_script);
            (command, None)
        },
        Runtime::Python => {
            let mut command = tokio::process::Command::new("python3");
            command.arg("-c");
            command.arg(templated_script);
            (command, None)
        },
        Runtime::Node => {
            let mut command = tokio::process::Command::new("node");
            command.arg("--input-type");
            command.arg("module");
            command.arg("-e");
            command.arg(templated_script);
            (command, None)
        },
        Runtime::Deno => {
            let mut command = tokio::process::Command::new("deno");
            command.arg("run");
            command.arg("-A");
            command.arg("-");
            command.stdin(Stdio::piped());

            (command, Some(templated_script))
        },
    };

    command.env("FIG_SCRIPT_EXECUTION", "1");

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(text) = text {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(text.as_bytes()).await?;
        stdin.flush().await?;
    }

    let mut stdout = child.stdout.take().unwrap();
    let stdout_join = tokio::spawn(async move {
        let mut stdout_buffer = BytesMut::new();
        loop {
            match stdout.read_buf(&mut stdout_buffer).await {
                Ok(0) => break,
                Ok(bytes) => {
                    let mut stdout = std::io::stdout().lock();
                    stdout.write_all(&stdout_buffer[stdout_buffer.len() - bytes..]).ok();
                    stdout.flush().ok();
                },
                Err(_) => break,
            }
        }
        stdout_buffer.freeze()
    });

    let mut stderr = child.stderr.take().unwrap();
    let stderr_join = tokio::spawn(async move {
        let mut stderr_buffer = BytesMut::new();
        loop {
            match stderr.read_buf(&mut stderr_buffer).await {
                Ok(0) => break,
                Ok(bytes) => {
                    let mut stderr = std::io::stderr().lock();
                    stderr.write_all(&stderr_buffer[stderr_buffer.len() - bytes..]).ok();
                    stderr.flush().ok();
                },
                Err(_) => break,
            }
        }
        stderr_buffer.freeze()
    });

    let runtime_version = {
        let runtime = script.runtime.clone();
        tokio::spawn(async move { runtime.version().await })
    };

    let inputs = args
        .iter()
        .map(|(k, v)| {
            (k, match v {
                Value::String(s) => serde_json::Value::String(s.clone()),
                Value::Bool { val, .. } => serde_json::Value::Bool(*val),
            })
        })
        .collect::<HashMap<_, _>>();

    tokio::select! {
        _res = tokio::signal::ctrl_c() => {
            child.kill().await?;

            eprintln!();
            eprintln!("{} script cancelled", format!("@{}/{}", script.namespace, script.name).magenta().bold());

            let execution_start_time = start_time.format(&Rfc3339).ok();
            let execution_duration = i64::try_from((OffsetDateTime::now_utc() - start_time).whole_nanoseconds()).ok();

            let stdout = stdout_join.await.ok().map(|b| String::from_utf8_lossy(&b).into_owned());
            let stderr = stderr_join.await.ok().map(|b| String::from_utf8_lossy(&b).into_owned());



            Request::post(format!("/workflows/{}/invocations", script.name))
                .body(serde_json::json!({
                    "namespace": script.namespace,
                    "executionStartTime": execution_start_time,
                    "executionDuration": execution_duration,
                    "ctrlC": true,
                    "runtimeVersion": runtime_version.await.ok().flatten(),
                    "inputs": script.invocation_track_inputs.then_some(inputs),
                    "stdout": script.invocation_track_stdout.then_some(stdout).flatten(),
                    "stderr": script.invocation_track_stderr.then_some(stderr).flatten(),
                }))
                .auth()
                .send()
                .await
                .ok();

            std::process::exit(130);
        },
        res = child.wait() => {
            let exit_code = res.ok().and_then(|output| output.code());

            let execution_start_time = start_time.format(&Rfc3339).ok();
            let execution_duration = i64::try_from((OffsetDateTime::now_utc() - start_time).whole_nanoseconds()).ok();

            let stdout = stdout_join.await.ok().map(|b| String::from_utf8_lossy(&b).into_owned());
            let stderr = stderr_join.await.ok().map(|b| String::from_utf8_lossy(&b).into_owned());

            Request::post(format!("/workflows/{}/invocations", script.name))
                .body(serde_json::json!({
                    "namespace": script.namespace,
                    "executionStartTime": execution_start_time,
                    "executionDuration": execution_duration,
                    "exitCode": exit_code,
                    "runtimeVersion": runtime_version.await.ok().flatten(),
                    "inputs": script.invocation_track_inputs.then_some(inputs),
                    "stdout": script.invocation_track_stdout.then_some(stdout).flatten(),
                    "stderr": script.invocation_track_stderr.then_some(stderr).flatten(),
                }))
                .auth()
                .send()
                .await
                .ok();
            Ok(())
        }
    }
}

/// Uses the setting `scripts.insert-into-shell` to determine whether to insert the command into the
/// shell or execute it directly
async fn execute_script_or_insert(script: &Script, args: &HashMap<String, Value>) -> Result<()> {
    if fig_settings::settings::get_bool_or("scripts.insert-into-shell", true)
        && std::env::var_os("FIG_SCRIPT_EXECUTION").is_none()
    {
        if send_figterm(map_args_to_command(script, args), true).await.is_err() {
            execute_script(script, args).await?;
        }
    } else {
        execute_script(script, args).await?;
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
            caret_color: ColorAttribute::PaletteIndex(3);
            color: ColorAttribute::PaletteIndex(7);
        },
        "div" => {
            width: Some(100.0);
        },
        "input:text", "input:checkbox", "select", "#__preview" => {
            padding_left: 1.0;
            padding_right: 1.0;
        },
        "#__parameter", "#__preview" => {
            border_left_width: 1.0;
            border_top_width: 1.0;
            border_bottom_width: 1.0;
            border_right_width: 1.0;
            border_left_color: ColorAttribute::PaletteIndex(8);
            border_right_color: ColorAttribute::PaletteIndex(8);
            border_top_color: ColorAttribute::PaletteIndex(8);
            border_bottom_color: ColorAttribute::PaletteIndex(8);
            border_style: BorderStyle::Ascii { top_left: '┌', top: '─', top_right: '┐', left: '│', right: '│', bottom_left: '└', bottom: '─', bottom_right: '┘' };
            padding_top: -1.0;
        },
        "#__parameter:focus" => {
            border_left_color: ColorAttribute::PaletteIndex(3);
            border_right_color: ColorAttribute::PaletteIndex(3);
            border_top_color: ColorAttribute::PaletteIndex(3);
            border_bottom_color: ColorAttribute::PaletteIndex(3);
        },
        "#__view" => {
            margin_left: 2.0;
            margin_right: 2.0;
            margin_top: 1.0;
            margin_bottom: 1.0;
        },
        "#__header" => {
            margin_bottom: 1.0;
        },
        "#__footer" => {
            margin_top: 1.0;
        },
        "#__label" => {
            padding_left: 1.0;
            padding_right: 1.0;
            margin_left: 1.0;
        }
    };

    let mut view = Container::new("__view", Layout::Vertical);

    let mut header = Paragraph::new("__header")
        .push_styled_text(
            script.display_name.as_ref().unwrap_or(&script.name),
            ColorAttribute::PaletteIndex(3),
            ColorAttribute::Default,
            true,
            false,
        )
        .push_styled_text(
            format!(" • @{}", script.namespace),
            ColorAttribute::Default,
            ColorAttribute::Default,
            false,
            false,
        );

    if let Some(description) = &script.description {
        if !description.is_empty() {
            header = header.push_styled_text(
                format!("\n{description}"),
                ColorAttribute::PaletteIndex(8),
                ColorAttribute::Default,
                false,
                true,
            );
        }
    }

    view.push(header);

    let mut form = Container::new("__form", Layout::Vertical);

    let mut args: HashMap<String, Value> = HashMap::new();
    let mut description_map = HashMap::new();
    let mut flag_map: HashMap<String, (String, String)> = HashMap::new();
    for parameter in &script.parameters {
        if let Some(description) = &parameter.description {
            description_map.insert(parameter.name.to_owned(), description.to_owned());
        }

        let mut parameter_div = Container::new("__parameter", Layout::Vertical);
        parameter_div
            .push(Paragraph::new("__label").push_text(parameter.display_name.as_ref().unwrap_or(&parameter.name)));

        match &parameter.parameter_type {
            ParameterType::Selector {
                placeholder,
                suggestions,
                generators,
            } => {
                let parameter_value = arg_pairs.get(&parameter.name).cloned().unwrap_or_default();
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

                parameter_div.push(
                    Select::new(&parameter.name, options, true)
                        .with_text(parameter_value)
                        .with_hint(placeholder.as_deref().unwrap_or("Search...")),
                );
            },
            ParameterType::Text { placeholder } => {
                let parameter_value = arg_pairs.get(&parameter.name).cloned().unwrap_or_default();
                parameter_div.push(
                    TextField::new(&parameter.name)
                        .with_text(parameter_value.to_string())
                        .with_hint(placeholder.to_owned().unwrap_or_default()),
                );
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

                parameter_div.push(CheckBox::new(
                    &parameter.name,
                    parameter.description.clone().unwrap_or_else(|| "Toggle".into()),
                    checked,
                ));
            },
            ParameterType::Path { file_type, extensions } => {
                let parameter_value = arg_pairs
                    .get(&parameter.name)
                    // TODO(chay): Fix this so it can be any file
                    .and_then(|val| {
                        let p = Path::new(val);
                        if p.is_dir() { Some(p.to_owned()) } else { None }
                    })
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| Path::new("/").to_owned());

                let (files, folders) = match file_type {
                    FileType::Any => (true, true),
                    FileType::FileOnly => (true, false),
                    FileType::FolderOnly => (false, true),
                };

                parameter_div.push(FilePicker::new(
                    &parameter.name,
                    parameter_value,
                    files,
                    folders,
                    extensions.clone(),
                ));
            },
        };

        form.push(parameter_div);
    }

    #[rustfmt::skip]
    view.push(form).push(
        Paragraph::new("__footer")
            .push_styled_text("enter", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text(" select • ", ColorAttribute::Default, ColorAttribute::Default, false, false)
            .push_styled_text("tab", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text(" next • ", ColorAttribute::Default, ColorAttribute::Default, false, false)
            .push_styled_text("shift+tab", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text(" previous • ", ColorAttribute::Default, ColorAttribute::Default, false, false)
            .push_styled_text("⎵", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text(" toggle • ", ColorAttribute::Default, ColorAttribute::Default, false, false)
            .push_styled_text( "⌃o", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text( " preview", ColorAttribute::Default, ColorAttribute::Default, false, false),
    );

    let mut temp = None;
    let mut terminated = false;

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
                            false,
                        ))
                        .ok();
                })
                .join()
                .ok();

                terminated = true;
                *control_flow = ControlFlow::Quit;
            },
            Event::TempChangeView => {
                if let Some(temp) = temp.take() {
                    view.replace("__preview", temp);
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
                                colors[hash % colors.len()],
                                ColorAttribute::Default,
                                false,
                                false,
                            );
                        },
                    }
                }

                let mut preview = Container::new("__preview", Layout::Vertical);
                preview
                    .push(Paragraph::new("__label").push_text("Preview"))
                    .push(paragraph);

                temp = view.remove("__form");
                view.insert("__header", Box::new(preview));
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

    if terminated {
        std::process::exit(1);
    }

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
