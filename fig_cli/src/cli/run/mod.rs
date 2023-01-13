#[cfg(unix)]
mod script_action;

use std::borrow::Cow;
use std::collections::{
    HashMap,
    HashSet,
};
use std::iter::empty;
use std::process::{
    Command,
    Stdio,
};
use std::time::Duration;

use clap::{
    ArgGroup,
    Args,
};
use crossterm::style::Stylize;
use eyre::{
    bail,
    Result,
};
use fig_api_client::scripts::{
    sync_scripts,
    FileType,
    Generator,
    Parameter,
    ParameterCommandlineInterfaceType,
    ParameterType,
    Predicate,
    Rule,
    RuleType,
    Runtime,
    Script,
    ScriptStep,
    TreeElement,
};
#[cfg(unix)]
use fig_ipc::local::open_ui_element;
#[cfg(unix)]
use fig_proto::local::UiElement;
use fig_telemetry::{
    TrackEvent,
    TrackEventType,
    TrackSource,
};
use fig_util::consts::FIG_SCRIPTS_SCHEMA_VERSION;
use fig_util::directories;
use time::OffsetDateTime;
use tokio::io::AsyncWriteExt;
use tokio::join;
use tokio::task::JoinHandle;
use tracing::error;
use tui::component::{
    CheckBox,
    CheckBoxEvent,
    Component,
    Div,
    FilePicker,
    FilePickerEvent,
    Select,
    SelectEvent,
    TextField,
    TextFieldEvent,
    P,
};
use tui::{
    ColorAttribute,
    ControlFlow,
    DisplayMode,
    Event,
    EventLoop,
    InputMethod,
    ParserOptions,
    StyleSheet,
};
use which::which;

#[cfg(unix)]
use crate::cli::run::script_action::ScriptAction;
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

async fn get_scripts(
    get_scripts_join: &mut Option<tokio::task::JoinHandle<fig_request::Result<Vec<fig_api_client::scripts::Script>>>>,
) -> Result<Vec<Script>> {
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
                    Err(err) => {
                        tracing::error!(%err, "Failed to parse script");

                        // If any file fails to parse, we should re-sync the scripts
                        if let Some(scripts_join) = get_scripts_join.take() {
                            return Ok(scripts_join.await??);
                        }
                    },
                }
            }
        }
    }

    Ok(scripts)
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
enum ParameterValue {
    String(String),
    Bool {
        val: bool,
        false_value: Option<String>,
        true_value: Option<String>,
    },
    Array(Vec<String>),
    Number(serde_json::Number),
}

impl std::fmt::Display for ParameterValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(val) => write!(f, "{val}"),
            Self::Bool {
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
            Self::Array(arr) => {
                let arr = arr.join(", ");
                write!(f, "[{arr}]")
            },
            Self::Number(num) => write!(f, "{num}"),
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

pub async fn execute(args: Vec<String>) -> Result<()> {
    let mut join_write_scripts = Some(tokio::spawn(sync_scripts()));
    let scripts = get_scripts(&mut join_write_scripts).await?;

    let interactive = atty::is(atty::Stream::Stdin)
        && atty::is(atty::Stream::Stdout)
        && std::env::var_os("FIG_SCRIPT_EXECUTION").is_none();

    let (execution_method, script) = match args.first().map(String::from) {
        Some(script_name) => get_named_script(scripts, &script_name, &mut join_write_scripts).await?,
        None => match search_over_scripts(scripts, interactive, &mut join_write_scripts).await? {
            Some(script) => script,
            None => return Ok(()),
        },
    };

    if std::env::var_os("FIG_SCRIPT_DEBUG").is_some() {
        println!("Script: {script:?}");
    }

    let script_name = format!("@{}/{}", &script.namespace, &script.name);
    if script.template_version > FIG_SCRIPTS_SCHEMA_VERSION {
        bail!(
            "Could not execute {script_name} since it requires features not available in this version of Fig.\n\
            Please update to the latest version by running {} and try again.",
            "fig update".magenta(),
        );
    }

    // Validate the script before tui, but after cli
    let validation = || async {
        // Validate that each runtime is installed
        for step in &script.steps {
            if let ScriptStep::CodeBlock { runtime, .. } = step {
                runtime_check(runtime).await?;
            }
        }

        // validate that all of the script rules pass
        if let Some(ruleset) = &script.rules {
            rules_check(ruleset)?;
        }

        Ok::<(), eyre::Error>(())
    };

    if args.len() > 1 {
        // If the user attempts to pass all their args on the cli, we must execute without prompt
        execute_from_cli(&script, &script_name, args).await?;
        validation().await?;
    } else {
        // Execute the script, which will exit internally on failure
        validation().await?;
        let mut parameters_by_name = HashMap::new();
        execute_script(&script, &mut parameters_by_name, execution_method).await?;
    }

    // Update the cache after script execution
    if let Some(write_scripts) = join_write_scripts.take() {
        write_scripts.await?.ok();
    }

    Ok(())
}

async fn get_named_script(
    mut scripts: Vec<Script>,
    name: &str,
    join_write_scripts: &mut Option<JoinHandle<Result<Vec<Script>, fig_request::Error>>>,
) -> Result<(ExecutionMethod, Script)> {
    let (namespace, name) = match name.strip_prefix('@') {
        Some(name) => match name.split('/').collect::<Vec<&str>>()[..] {
            [namespace, name] => (Some(namespace), name),
            _ => bail!("Malformed script specifier, expects '@namespace/script-name': {name}",),
        },
        None => (None, name),
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
            scripts = get_scripts(join_write_scripts).await?;

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
                None => bail!("Script not found"),
            }
        },
    };

    Ok((ExecutionMethod::Invoke, script))
}

async fn search_over_scripts(
    mut scripts: Vec<Script>,
    interactive: bool,
    join_write_scripts: &mut Option<JoinHandle<Result<Vec<Script>, fig_request::Error>>>,
) -> Result<Option<(ExecutionMethod, Script)>> {
    if !interactive {
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
                tx.send(Arc::new(script_action::ScriptAction::Create)).ok();
            }

            for script in scripts.iter().rev() {
                tx.send(Arc::new(script_action::ScriptAction::Run(Box::new(script.clone())))).ok();
            }
            drop(tx);

            let terminal_height = crossterm::terminal::size()?.1;
            let mut cursor_position = crossterm::cursor::position().unwrap_or((0, 0));

            let height = (terminal_height - cursor_position.1).max(13);
            let remaining_height = terminal_height.saturating_sub(cursor_position.1);
            let needed_height = height.saturating_sub(remaining_height);
            cursor_position.1 = cursor_position.1.saturating_sub(needed_height);
            let height = height.to_string();

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

            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(cursor_position.0, cursor_position.1),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
            )?;

            let script = match output {
                Some(SkimOutput { is_abort: true, .. }) => None,
                Some(output) => {
                    match output.selected_items.iter()
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
                                ScriptAction::Run(script) => Some(*script.clone()),
                                ScriptAction::Create => {
                                    launch_fig_desktop(LaunchArgs {
                                        wait_for_socket: true,
                                        open_dashboard: false,
                                        immediate_update: true,
                                        verbose: true,
                                    })?;

                                    open_ui_element(UiElement::MissionControl, Some("/scripts".to_string())).await?;
                                    return Ok(None)
                                },
                            }
                        }
                        None => None,
                    }
                },
                None => None,
            };
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

            let script = scripts.remove(selection);
        }
    }

    match script {
        Some(mut script) => {
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
                        None => bail!("Script is no longer available"),
                    }
                }
            }

            Ok(Some((ExecutionMethod::Search, script)))
        },
        None => Ok(None),
    }
}

async fn execute_from_cli(script: &Script, script_name: &str, args: Vec<String>) -> Result<()> {
    let mut command = clap::Command::new(script_name.to_owned());
    if let Some(description) = &script.description {
        command = command.about(description);
    }

    for step in &script.steps {
        if let ScriptStep::Inputs { parameters, .. } = step {
            for parameter in parameters {
                match parameter.cli.as_ref().and_then(|cli| cli.r#type.as_ref()) {
                    Some(param_type) => {
                        let mut arg = clap::Arg::new(&parameter.name);

                        match &parameter.cli {
                            Some(interface) => {
                                if let Some(short) = &interface.short {
                                    if let Some(first_char) = short.chars().next() {
                                        arg = arg.short(first_char);
                                    }
                                }

                                if let Some(long) = &interface.long {
                                    arg = arg.long(long);
                                }

                                if let Some(require_equals) = &interface.require_equals {
                                    arg = arg.require_equals(*require_equals);
                                }

                                if let Some(raw) = &interface.raw {
                                    arg = arg.raw(*raw);
                                }

                                arg = arg.required(interface.required.unwrap_or(true));
                            },
                            None => {
                                arg = arg.long(&parameter.name);

                                if parameter.name.len() == 1 {
                                    arg = arg.short(parameter.name.chars().next().unwrap());
                                }

                                arg = arg.required(true);
                            },
                        };

                        if let Some(description) = &parameter.description {
                            arg = arg.help(description);
                        }

                        match param_type {
                            ParameterCommandlineInterfaceType::Boolean { .. } => {
                                arg = arg.value_parser(clap::value_parser!(bool));
                            },
                            ParameterCommandlineInterfaceType::String { default } => {
                                arg = arg.value_parser(clap::value_parser!(String));

                                if let Some(default) = default {
                                    arg = arg.default_value(default.to_string());
                                }
                            },
                        }

                        command = command.arg(arg);
                    },
                    None => {
                        match &parameter.parameter_type {
                            ParameterType::Text { .. }
                            | ParameterType::Path { .. }
                            | ParameterType::Selector { .. } => {
                                let mut arg = clap::Arg::new(&parameter.name);

                                match &parameter.cli {
                                    Some(interface) => {
                                        if let Some(short) = &interface.short {
                                            if let Some(first_char) = short.chars().next() {
                                                arg = arg.short(first_char);
                                            }
                                        }

                                        if let Some(long) = &interface.long {
                                            arg = arg.long(long);
                                        }

                                        if let Some(require_equals) = &interface.require_equals {
                                            arg = arg.require_equals(*require_equals);
                                        }

                                        if let Some(raw) = &interface.raw {
                                            arg = arg.raw(*raw);
                                        }

                                        arg = arg.required(interface.required.unwrap_or(true));
                                    },
                                    None => {
                                        arg = arg.long(&parameter.name);

                                        if parameter.name.len() == 1 {
                                            arg = arg.short(parameter.name.chars().next().unwrap());
                                        }

                                        arg = arg.required(true);
                                    },
                                };

                                if let Some(description) = &parameter.description {
                                    arg = arg.help(description);
                                }

                                command = command.arg(arg.value_parser(clap::value_parser!(String)));
                            },
                            ParameterType::Checkbox { .. } => {
                                let required = match &parameter.cli {
                                    Some(interface) => interface.required.unwrap_or(true),
                                    None => true,
                                };

                                command = command.group(
                                    ArgGroup::new(format!("_{}_group", parameter.name))
                                        .arg(&parameter.name)
                                        .arg(format!("no-{}", &parameter.name))
                                        .required(required)
                                        .multiple(false),
                                );

                                let mut true_arg = clap::Arg::new(&parameter.name)
                                    .long(&parameter.name)
                                    .action(clap::ArgAction::SetTrue);

                                if let Some(description) = &parameter.description {
                                    true_arg = true_arg.help(description);
                                }

                                command = command.arg(true_arg);

                                let mut false_arg = clap::Arg::new(format!("no-{}", &parameter.name))
                                    .long(format!("no-{}", &parameter.name))
                                    .action(clap::ArgAction::SetFalse);

                                if let Some(description) = &parameter.description {
                                    false_arg = false_arg.help(description);
                                }

                                command = command.arg(false_arg);
                            },
                            ParameterType::Unknown(unknown) => {
                                bail!("Unknown parameter type, you may need to update Fig: {unknown:?}")
                            },
                        };
                    },
                }
            }
        }
    }

    let mut matches = command.get_matches_from(args);

    let mut parameters_by_name = HashMap::new();
    for step in &script.steps {
        if let ScriptStep::Inputs { parameters, .. } = step {
            for parameter in parameters {
                match &parameter.parameter_type {
                    ParameterType::Selector { .. } | ParameterType::Text { .. } | ParameterType::Path { .. } => {
                        if let Some(value) = matches.remove_one::<String>(&parameter.name) {
                            parameters_by_name.insert(parameter.name.clone(), ParameterValue::String(value));
                        }
                    },
                    ParameterType::Checkbox {
                        false_value_substitution,
                        true_value_substitution,
                    } => {
                        parameters_by_name.insert(parameter.name.clone(), ParameterValue::Bool {
                            val: matches.get_flag(&parameter.name),
                            false_value: Some(false_value_substitution.clone()),
                            true_value: Some(true_value_substitution.clone()),
                        });
                    },
                    ParameterType::Unknown(other) => {
                        bail!("Unknown parameter type, you may need to update Fig: {other:?}")
                    },
                };
            }
        }
    }

    execute_script(script, &mut parameters_by_name, ExecutionMethod::Invoke).await
}

fn interpolate_ast(runtime: Runtime, tree: &[TreeElement], args: &HashMap<String, ParameterValue>) -> String {
    tree.iter().fold(String::new(), |mut acc, branch| {
        match branch {
            TreeElement::String(string) => acc.push_str(string.as_str()),
            TreeElement::Token { name } => acc.push_str(&match args.get(name.as_str()) {
                Some(ParameterValue::String(string)) => match runtime {
                    Runtime::Bash => string.clone(),
                    Runtime::Python | Runtime::Node | Runtime::Deno => {
                        serde_json::to_string(string).expect("Failed to serialize string to JSON string")
                    },
                },
                Some(ParameterValue::Bool {
                    val,
                    true_value,
                    false_value,
                }) => match (&runtime, val) {
                    (Runtime::Bash, true) => true_value.clone().unwrap_or_else(|| "true".into()),
                    (Runtime::Bash, false) => false_value.clone().unwrap_or_else(|| "false".into()),
                    (Runtime::Python, true) => "True".into(),
                    (Runtime::Python, false) => "False".into(),
                    (Runtime::Node | Runtime::Deno, true) => "true".into(),
                    (Runtime::Node | Runtime::Deno, false) => "false".into(),
                },
                Some(ParameterValue::Array(arr)) => match &runtime {
                    Runtime::Bash => {
                        let mut out: String = "(".into();
                        for (i, s) in arr.iter().enumerate() {
                            if i != 0 {
                                out.push(' ');
                            }
                            out.push_str(&escape(s.into()));
                        }
                        out.push(')');
                        out
                    },
                    Runtime::Python | Runtime::Node | Runtime::Deno => {
                        serde_json::to_string(arr).expect("Failed to serialize array to JSON string")
                    },
                },
                Some(ParameterValue::Number(num)) => num.to_string(),
                None => match runtime {
                    Runtime::Bash => "\"\"".into(),
                    Runtime::Python => "None".into(),
                    Runtime::Node | Runtime::Deno => "null".into(),
                },
            }),
        }

        acc
    })
}

#[derive(Debug, Clone)]
struct ScriptGeneratorState {
    tree: Vec<TreeElement>,
    last_execution: Option<String>,
    results: Option<Vec<String>>,
    depends_on: HashSet<String>,
}

impl ScriptGeneratorState {
    fn from_tree(tree: Vec<TreeElement>) -> Self {
        Self {
            tree: tree.clone(),
            last_execution: None,
            results: None,
            depends_on: tree
                .iter()
                .filter_map(|branch| match branch {
                    TreeElement::Token { name } => Some(name.to_owned()),
                    _ => None,
                })
                .collect(),
        }
    }

    fn execute(&mut self, args: &HashMap<String, ParameterValue>) -> bool {
        let script = interpolate_ast(Runtime::Bash, &self.tree, args);
        let should_run = self
            .last_execution
            .as_ref()
            .map(|prev| prev.as_str() != script)
            .unwrap_or(true);

        if should_run {
            if let Ok(output) = Command::new("bash").arg("-c").arg(&script).output() {
                let mut options = vec![];
                for suggestion in String::from_utf8_lossy(&output.stdout).split('\n') {
                    if !suggestion.is_empty() {
                        options.push(suggestion.to_owned());
                    }
                }
                self.results = Some(options);
                self.last_execution = Some(script);
            }
            true
        } else {
            false
        }
    }
}

async fn execute_script(
    script: &Script,
    parameters_by_name: &mut HashMap<String, ParameterValue>,
    execution_method: ExecutionMethod,
) -> Result<()> {
    let start_time = time::OffsetDateTime::now_utc();

    let mut exit_code = None;
    for step in &script.steps {
        if let Some(code) = execute_step(step, parameters_by_name).await? {
            exit_code = Some(code);
            break;
        }
    }

    let execution_duration = u64::try_from((OffsetDateTime::now_utc() - start_time).whole_nanoseconds()).ok();

    let script_uuid = script.uuid.clone();
    let script_name = format!("@{}/{}", &script.namespace, &script.name);
    let telem_join = tokio::spawn(fig_telemetry::dispatch_emit_track(
        TrackEvent::new(
            TrackEventType::ScriptExecuted,
            TrackSource::Cli,
            env!("CARGO_PKG_VERSION").into(),
            [
                ("workflow", script_name.as_str()),
                ("script_uuid", &script_uuid),
                ("execution_method", execution_method.to_string().as_str()),
            ],
        )
        .with_namespace(Some(script.namespace.clone())),
        false,
        true,
    ));

    match script.invocation_disable_track {
        true => {
            telem_join.await.ok();
        },
        false => {
            let query = fig_graphql::create_script_invocation_query!(
                name: script.name.clone(),
                namespace: script.namespace.clone(),
                execution_start_time: Some(start_time.into()),
                execution_duration: execution_duration,
                ..Default::default(),
            );

            let (_, invocation) = join!(telem_join, fig_graphql::dispatch::send_to_daemon(query, true));

            if let Err(err) = invocation {
                error!(%err, "Failed to create script invocation");
            }
        },
    }

    if let Some(exit_code) = exit_code {
        std::process::exit(exit_code);
    }

    Ok(())
}

async fn execute_step(
    step: &ScriptStep,
    parameters_by_name: &mut HashMap<String, ParameterValue>,
) -> Result<Option<i32>> {
    match step {
        ScriptStep::Inputs { name, parameters } => execute_parameter_block(parameters_by_name, name, parameters)?,
        ScriptStep::CodeBlock { runtime, tree, .. } => {
            return execute_code_block(parameters_by_name, runtime, tree).await.map(Some);
        },
    }

    Ok(None)
}

fn execute_parameter_block(
    parameters_by_name: &mut HashMap<String, ParameterValue>,
    name: &Option<String>,
    parameters: &Vec<Parameter>,
) -> Result<()> {
    if parameters
        .iter()
        .all(|parameter| parameters_by_name.contains_key(&parameter.name))
    {
        return Ok(());
    }

    let mut view = Div::new().with_class("view");

    if let Some(name) = name {
        view = view.push(P::new().push_styled_text(
            name,
            ColorAttribute::PaletteIndex(3),
            ColorAttribute::Default,
            true,
            false,
        ))
    }

    let mut parameter_map: HashMap<String, &Parameter> = HashMap::new();
    let mut generator_map: HashMap<String, Vec<ScriptGeneratorState>> = HashMap::new();
    let mut parameter_dependencies: HashMap<String, HashSet<String>> = HashMap::new();

    for parameter in parameters {
        parameter_map.insert(parameter.name.to_owned(), parameter);

        let mut parameter_label = P::new()
            .with_class("label")
            .push_text(parameter.display_name.as_ref().unwrap_or(&parameter.name));

        if !parameter.required.unwrap_or(true) {
            parameter_label = parameter_label.push_styled_text(
                " - Optional",
                ColorAttribute::PaletteIndex(8),
                ColorAttribute::Default,
                false,
                true,
            );
        }

        let mut parameter_div = Div::new().with_class("parameter").push(parameter_label);

        let keybindings = match &parameter.parameter_type {
            ParameterType::Selector {
                placeholder,
                suggestions,
                generators,
                allow_raw_text_input,
            } => {
                let parameter_value = parameters_by_name
                    .get(&parameter.name)
                    .map(|parameter| parameter.to_string())
                    .unwrap_or_default();

                let options = suggestions.to_owned().unwrap_or_default();
                if let Some(generators) = generators {
                    let mut generator_states = vec![];
                    for generator in generators {
                        let state = match generator {
                            Generator::Script { tree, .. } => ScriptGeneratorState::from_tree(tree.clone()),
                            Generator::Named { .. } => bail!("Named generators aren't supported in scripts yet"),
                            Generator::Unknown(unknown) => {
                                bail!("Unknown generator type, try updating your Fig version: {unknown:?}")
                            },
                        };
                        generator_states.push(state);
                    }
                    generator_map.insert(parameter.name.to_owned(), generator_states);
                }

                parameter_div = parameter_div.push(
                    Select::new(options, allow_raw_text_input.unwrap_or(false))
                        .with_id(&parameter.name)
                        .with_text(parameter_value)
                        .with_hint(placeholder.as_deref().unwrap_or("Search...")),
                );

                Some(
                    P::new()
                        .with_class("keybindings")
                        .push_styled_text(
                            "↑/↓",
                            ColorAttribute::PaletteIndex(3),
                            ColorAttribute::Default,
                            false,
                            false,
                        )
                        .push_styled_text(
                            " up/down",
                            ColorAttribute::Default,
                            ColorAttribute::Default,
                            false,
                            false,
                        ),
                )
            },
            ParameterType::Text { placeholder } => {
                let parameter_value = parameters_by_name
                    .get(&parameter.name)
                    .map(|parameter| parameter.to_string())
                    .unwrap_or_default();

                parameter_div = parameter_div.push(
                    TextField::new()
                        .with_id(&parameter.name)
                        .with_text(parameter_value.to_string())
                        .with_hint(placeholder.to_owned().unwrap_or_default()),
                );

                None
            },
            ParameterType::Checkbox {
                true_value_substitution,
                false_value_substitution,
            } => {
                let checked = parameters_by_name
                    .get(&parameter.name)
                    .map(|val| match val {
                        ParameterValue::Bool { val, .. } => *val,
                        _ => false,
                    })
                    .unwrap_or(false);

                parameters_by_name.insert(parameter.name.to_owned(), ParameterValue::Bool {
                    val: checked,
                    true_value: Some(true_value_substitution.to_owned()),
                    false_value: Some(false_value_substitution.to_owned()),
                });

                parameter_div = parameter_div.push(CheckBox::new("Toggle", checked).with_id(&parameter.name));

                Some(
                    P::new()
                        .with_class("keybindings")
                        .push_styled_text(
                            "⎵",
                            ColorAttribute::PaletteIndex(3),
                            ColorAttribute::Default,
                            false,
                            false,
                        )
                        .push_styled_text(
                            " toggle",
                            ColorAttribute::Default,
                            ColorAttribute::Default,
                            false,
                            false,
                        ),
                )
            },
            ParameterType::Path { file_type, extensions } => {
                let cwd = std::env::current_dir()
                    .ok()
                    .map(|cwd| cwd.to_str().unwrap_or("/").to_owned())
                    .unwrap_or_else(|| "/".to_owned());

                let parameter_value = parameters_by_name
                    .get(&parameter.name)
                    .map(|inner| inner.to_string())
                    .unwrap_or_else(|| cwd.clone());

                let (files, folders) = match file_type {
                    FileType::Any | FileType::Unknown(_) => (true, true),
                    FileType::FileOnly => (true, false),
                    FileType::FolderOnly => (false, true),
                };

                parameters_by_name.insert(parameter.name.to_owned(), ParameterValue::String(cwd));

                parameter_div = parameter_div.push(
                    FilePicker::new(files, folders, extensions.clone())
                        .with_id(&parameter.name)
                        .with_path(parameter_value),
                );

                Some(
                    P::new()
                        .with_class("keybindings")
                        .push_styled_text(
                            "↑/↓",
                            ColorAttribute::PaletteIndex(3),
                            ColorAttribute::Default,
                            false,
                            false,
                        )
                        .push_styled_text(
                            " up/down • ",
                            ColorAttribute::Default,
                            ColorAttribute::Default,
                            false,
                            false,
                        )
                        .push_styled_text(
                            "←/→",
                            ColorAttribute::PaletteIndex(3),
                            ColorAttribute::Default,
                            false,
                            false,
                        )
                        .push_styled_text(
                            " traverse",
                            ColorAttribute::Default,
                            ColorAttribute::Default,
                            false,
                            false,
                        ),
                )
            },
            ParameterType::Unknown(other) => {
                bail!("Unknown parameter type, you may need to update Fig: {other:?}")
            },
        };

        if let Some(description) = &parameter.description {
            parameter_div = parameter_div.push(
                P::new()
                    .with_class("description")
                    .push_styled_text(
                        "─".repeat(100),
                        ColorAttribute::PaletteIndex(8),
                        ColorAttribute::Default,
                        false,
                        true,
                    )
                    .push_styled_text(
                        format!("\n{description}"),
                        ColorAttribute::PaletteIndex(8),
                        ColorAttribute::Default,
                        false,
                        true,
                    ),
            );
        }

        if let Some(keybindings) = keybindings {
            parameter_div = parameter_div.push(keybindings);
        }

        view = view.push(parameter_div);
    }

    let initial_generators = generator_map.clone();
    let mut update_select_options_in_view =
        |id: &String, view: &mut dyn Component, arg_values: &HashMap<String, ParameterValue>| {
            let did_execute = match generator_map.get_mut(id) {
                Some(gens) => gens.iter_mut().any(|gen| gen.execute(arg_values)),
                None => false,
            };
            if did_execute {
                if let ParameterType::Selector { suggestions, .. } = &parameter_map.get(id).unwrap().parameter_type {
                    let mut options = suggestions.to_owned().unwrap_or_default();
                    for gen in generator_map.get(id).unwrap() {
                        options.extend(gen.results.clone().unwrap_or_default());
                    }
                    if let Some(select) = view.find_mut(id).and_then(|e| e.downcast_mut::<Select>()) {
                        select.set_options(options);
                    }
                }
            }
        };

    for (key, generator_states) in initial_generators.iter() {
        update_select_options_in_view(key, &mut view, parameters_by_name);
        let depends_on = generator_states.iter().fold(HashSet::new(), |mut acc, g| {
            acc.extend(g.depends_on.clone());
            acc
        });

        for name in depends_on {
            match parameter_dependencies.get_mut(&name) {
                Some(keys) => {
                    keys.insert(key.to_owned());
                },
                None => {
                    let keys = HashSet::from_iter(vec![key.to_owned()]);
                    parameter_dependencies.insert(name, keys);
                },
            }
        }
    }

    let mut selectors_pending_update = HashSet::new();

    #[rustfmt::skip]
    let view = view.push(
        P::new().with_class("footer")
            .push_styled_text("enter", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text(" select • ", ColorAttribute::Default, ColorAttribute::Default, false, false)
            .push_styled_text("tab", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text(" next • ", ColorAttribute::Default, ColorAttribute::Default, false, false)
            .push_styled_text("shift+tab", ColorAttribute::PaletteIndex(3), ColorAttribute::Default, false, false)
            .push_styled_text(" previous", ColorAttribute::Default, ColorAttribute::Default, false, false)
    );

    let mut terminated = false;
    #[allow(clippy::collapsible_match, clippy::single_match)]
    EventLoop::new(
        view,
        DisplayMode::Inline,
        InputMethod::new(),
        StyleSheet::parse(include_str!("run.css"), ParserOptions::default())?,
    )
    .run_with_timeout(
        Some(Duration::from_millis(100)),
        |event, view, control_flow| match event {
            Event::Quit => *control_flow = ControlFlow::Quit,
            Event::Terminate => {
                terminated = true;
                *control_flow = ControlFlow::Quit;
            },
            Event::CheckBox(event) => match event {
                CheckBoxEvent::Checked { id: Some(id), checked } => {
                    let param = parameter_map.get(&id).unwrap();
                    if let ParameterType::Checkbox {
                        ref true_value_substitution,
                        ref false_value_substitution,
                    } = param.parameter_type
                    {
                        if let Some(selectors) = parameter_dependencies.get(&id) {
                            selectors_pending_update.extend(selectors)
                        }
                        parameters_by_name.insert(id, ParameterValue::Bool {
                            val: checked,
                            false_value: Some(false_value_substitution.to_owned()),
                            true_value: Some(true_value_substitution.to_owned()),
                        });
                    }
                },
                _ => (),
            },
            Event::FilePicker(event) => match event {
                FilePickerEvent::FilePathChanged { id: Some(id), path } => {
                    if let Some(selectors) = parameter_dependencies.get(&id) {
                        selectors_pending_update.extend(selectors)
                    }
                    parameters_by_name.insert(id, ParameterValue::String(path.to_string_lossy().to_string()));
                },
                _ => (),
            },
            Event::Select(event) => match event {
                SelectEvent::OptionSelected { id: Some(id), option } => {
                    if let Some(selectors) = parameter_dependencies.get(&id) {
                        selectors_pending_update.extend(selectors)
                    }
                    parameters_by_name.insert(id, ParameterValue::String(option));
                },
                _ => (),
            },
            Event::TextField(event) => match event {
                TextFieldEvent::TextChanged { id: Some(id), text } => {
                    if let Some(selectors) = parameter_dependencies.get(&id) {
                        selectors_pending_update.extend(selectors)
                    }
                    parameters_by_name.insert(id, ParameterValue::String(text));
                },
                _ => (),
            },
            Event::MainEventsCleared => {
                for selector_id in selectors_pending_update.iter() {
                    update_select_options_in_view(selector_id.to_owned(), view, parameters_by_name);
                }
                selectors_pending_update.clear();
            },
            _ => (),
        },
    )?;

    if terminated {
        std::process::exit(1);
    }

    Ok(())
}

async fn execute_code_block(
    parameters_by_name: &mut HashMap<String, ParameterValue>,
    runtime: &Runtime,
    tree: &[TreeElement],
) -> Result<i32> {
    let templated_script = interpolate_ast(runtime.clone(), tree, parameters_by_name);

    let (mut command, text) = match runtime {
        Runtime::Bash => {
            let mut command = tokio::process::Command::new(runtime.exe());
            command.arg("-c");
            command.arg(templated_script);
            (command, None)
        },
        Runtime::Python => {
            let mut command = tokio::process::Command::new(runtime.exe());
            command.arg("-c");
            command.arg(templated_script);
            (command, None)
        },
        Runtime::Node => {
            let mut command = tokio::process::Command::new(runtime.exe());
            command.arg("--input-type");
            command.arg("module");
            command.arg("-e");
            command.arg(templated_script);
            (command, None)
        },
        Runtime::Deno => {
            let mut command = tokio::process::Command::new(runtime.exe());
            command.arg("run");
            command.arg("-A");
            command.arg("-");
            command.stdin(Stdio::piped());

            (command, Some(templated_script))
        },
    };

    command.env("FIG_SCRIPT_EXECUTION", "1");

    // command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(text) = text {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(text.as_bytes()).await?;
        stdin.flush().await?;
    }

    // let mut stdout = child.stdout.take().unwrap();
    // let stdout_join = tokio::spawn(async move {
    //    let mut stdout_buffer = BytesMut::new();
    //    loop {
    //        match stdout.read_buf(&mut stdout_buffer).await {
    //            Ok(0) => break,
    //            Ok(bytes) => {
    //                let mut stdout = std::io::stdout().lock();
    //                stdout.write_all(&stdout_buffer[stdout_buffer.len() - bytes..]).ok();
    //                stdout.flush().ok();
    //            },
    //            Err(_) => break,
    //        }
    //    }
    //    stdout_buffer.freeze()
    //});

    // let mut stderr = child.stderr.take().unwrap();
    // let stderr_join = tokio::spawn(async move {
    //    let mut stderr_buffer = BytesMut::new();
    //    loop {
    //        match stderr.read_buf(&mut stderr_buffer).await {
    //            Ok(0) => break,
    //            Ok(bytes) => {
    //                let mut stderr = std::io::stderr().lock();
    //                stderr.write_all(&stderr_buffer[stderr_buffer.len() - bytes..]).ok();
    //                stderr.flush().ok();
    //            },
    //            Err(_) => break,
    //        }
    //    }
    //    stderr_buffer.freeze()
    //});

    tokio::select! {
        res = tokio::signal::ctrl_c() => {
            res?;
            child.kill().await?;

            eprintln!();
            eprintln!("script cancelled");
            Ok(130)
        },
        res = child.wait() => Ok(res?.code().unwrap_or(0)),
    }
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
    let mut first_confirm = true;
    let mut confirm = |name: &str| {
        let install = matches!(
            choose(
                &format!(
                    "{}Would you like to install {runtime:?} with {name}?",
                    if first_confirm {
                        format!("{runtime:?} is not installed. ")
                    } else {
                        "".into()
                    }
                ),
                &["Yes", "No"],
            ),
            Ok(0)
        );
        first_confirm = false;
        install
    };

    let error = || {
        eyre::eyre!(
            "Failed to execute script, {runtime:?} is not installed{}",
            if let Some(install_docs) = runtime.install_docs() {
                format!(" (see {install_docs})")
            } else {
                "".into()
            }
        )
    };

    // if not interactive, don't try to install
    if !atty::is(atty::Stream::Stdout) {
        return Err(error());
    }

    #[cfg(target_os = "macos")]
    if which("brew").is_ok() && confirm("brew") {
        let mut command = Command::new("brew");
        command.arg("install");
        command.arg("--quiet");
        command.arg(runtime.brew_package());

        command.env("HOMEBREW_NO_AUTO_UPDATE", "1");
        command.env("HOMEBREW_NO_ENV_HINTS", "1");

        if !command.status()?.success() {
            bail!("Failed to install {runtime:?} with brew");
        }
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    if which("pacman").is_ok() && confirm("pacman") {
        let mut command = Command::new("sudo");
        command.arg("pacman");
        command.arg("-S");
        command.arg(runtime.pacman_package());

        if !command.status()?.success() {
            bail!("Failed to install {runtime:?} with pacman");
        };
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    if let Some(dnf_package) = runtime.dnf_package() {
        if which("dnf").is_ok() && confirm("dnf") {
            let mut command = Command::new("sudo");
            command.arg("dnf");
            command.arg("install");
            command.arg(dnf_package);

            if !command.status()?.success() {
                bail!("Failed to install {runtime:?} with dnf");
            };
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    if let Some(apt_package) = runtime.apt_package() {
        if which("apt-get").is_ok() && confirm("apt-get") {
            let mut command = Command::new("sudo");
            command.arg("apt-get");
            command.arg("install");
            command.arg(apt_package);

            if !command.status()?.success() {
                bail!("Failed to install {runtime:?} with apt-get");
            };
            return Ok(());
        }
    }

    #[cfg(unix)]
    if let Some(fallback_install_script) = runtime.fallback_install_script() {
        if which("bash").is_ok() && confirm(&format!("'{fallback_install_script}'")) {
            let mut command = Command::new("bash");
            command.arg("-c");
            command.arg(fallback_install_script);

            if !command.status()?.success() {
                bail!("Failed to install {runtime:?} with bash");
            };
            return Ok(());
        }
    }

    Err(error())
}

fn rules_check(ruleset: &Vec<Vec<Rule>>) -> Result<()> {
    for set in ruleset {
        let mut set_met = set.is_empty();
        for rule in set {
            let query = match &rule.key {
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
                RuleType::GitRootDirectory => {
                    let dir = Command::new("git")
                        .args(["rev-parse", "--show-toplevel"])
                        .output()?
                        .stdout;

                    match std::fs::read_dir(std::str::from_utf8(&dir)?) {
                        Ok(dir) => {
                            let mut out = String::new();
                            for file in dir {
                                out.push_str(&format!("\"{}\" ", &file?.path().to_string_lossy()));
                            }

                            out
                        },
                        Err(err) => bail!("Failed to read git root directory: {err}"),
                    }
                },
                RuleType::CurrentBranch => String::from_utf8(
                    Command::new("git")
                        .args(["rev-parse", "--abbrev-ref", "HEAD"])
                        .output()?
                        .stdout,
                )?,
                RuleType::EnvironmentVariable => bail!("Environment variable rules are not yet supported"),
                RuleType::Unknown(other) => bail!("Unknown rule, you may need to update fig: {other}"),
            };

            let query = query.trim();

            let Some(value) = rule.value.as_deref() else {
                bail!("Rule value is missing");
            };

            let mut rule_met = match &rule.predicate {
                Predicate::Contains => query.contains(value),
                Predicate::Equals => query == value,
                Predicate::Matches => regex::Regex::new(value)?.is_match(query),
                Predicate::StartsWith => query.starts_with(value),
                Predicate::EndsWith => query.ends_with(value),
                Predicate::Exists => !query.is_empty(),
                Predicate::Unknown(other) => bail!("Unknown predicate, you may need to update fig: {other}"),
            };

            if rule.inverted {
                rule_met = !rule_met;
            }

            set_met |= rule_met;
        }

        if !set_met {
            let mut err_message = format!(
                "{}",
                if set.len() == 1 {
                    "The following rule must be met:\n"
                } else {
                    "One of the following rules must be met:\n"
                }
                .red()
            );

            for rule in set {
                err_message.push_str(&format!("- {rule}\n"));
            }

            err_message.push('\n');

            bail!(err_message);
        }
    }

    Ok(())
}
