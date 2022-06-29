use std::borrow::Cow;
use std::collections::HashMap;
use std::process::Command;

use anyhow::{
    anyhow,
    bail,
    Result,
};
use clap::Args;
use crossterm::{execute, cursor};
use crossterm::style::Stylize;
use fig_ipc::command::open_ui_element;
use fig_proto::local::UiElement;
use fig_telemetry::{
    TrackEvent,
    TrackSource,
};
use reqwest::Method;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
#[cfg(unix)]
use skim::SkimItem;
use spinners::{
    Spinner,
    Spinners,
};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tui::components::{
    CheckBox,
    Frame,
    Label,
    Select,
    TextField,
};
use tui::layouts::Form;
use tui::{
    BorderStyle,
    Color,
    Component,
    ControlFlow,
    DisplayMode,
    EventLoop,
};

use crate::util::api::request;
use crate::util::{
    launch_fig,
    LaunchOptions,
};

const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Args)]
pub struct WorkflowArgs {
    // Flags can be added here
    #[clap(value_parser, takes_value = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

impl WorkflowArgs {
    pub async fn execute(self) -> Result<()> {
        execute(self.args).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum Generator {
    #[serde(rename_all = "camelCase")]
    Named { name: String },
    #[serde(rename_all = "camelCase")]
    Script { script: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "typeData")]
#[serde(rename_all = "camelCase")]
enum ParameterType {
    #[serde(rename_all = "camelCase")]
    Checkbox {
        true_value_substitution: String,
        false_value_substitution: String,
    },
    #[serde(rename_all = "camelCase")]
    Text { placeholder: Option<String> },
    #[serde(rename_all = "camelCase")]
    Selector {
        placeholder: Option<String>,
        suggestions: Option<Vec<String>>,
        generators: Option<Vec<Generator>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Parameter {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    parameter_type: ParameterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
enum TreeElement {
    String(String),
    Token { name: String },
}

#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Workflow {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    template_version: u32,
    tags: Option<Vec<String>>,
    parameters: Vec<Parameter>,
    namespace: String,
    template: String,
    tree: Vec<TreeElement>,
}

enum WorkflowAction {
    Run(Workflow),
    Create,
}

#[cfg(unix)]
impl SkimItem for WorkflowAction {
    fn text(&self) -> std::borrow::Cow<str> {
        match self {
            WorkflowAction::Run(workflow) => {
                let tags = match &workflow.tags {
                    Some(tags) => tags.join(" "),
                    None => String::new(),
                };

                format!(
                    "{} {} @{}/{} {}",
                    workflow.display_name.as_deref().unwrap_or_default(),
                    workflow.name,
                    workflow.namespace,
                    workflow.name,
                    tags
                )
                .into()
            },
            WorkflowAction::Create => "create new workflow".into(),
        }
    }

    fn display<'a>(&'a self, context: skim::DisplayContext<'a>) -> skim::AnsiString<'a> {
        match self {
            WorkflowAction::Run(workflow) => {
                let name = workflow.display_name.clone().unwrap_or_else(|| workflow.name.clone());
                let name_len = name.len();

                let tags = match &workflow.tags {
                    Some(tags) if !tags.is_empty() => format!(" |{}| ", tags.join("|")),
                    _ => String::new(),
                };
                let tag_len = tags.len();

                let namespace_name = format!("@{}/{}", workflow.namespace, workflow.name);
                let namespace_name_len = namespace_name.len();

                skim::AnsiString::parse(&format!(
                    "{}{}{}{}",
                    name.bold(),
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
            },
            WorkflowAction::Create => skim::AnsiString::parse(&"Create new Workflow...".bold().blue().to_string()),
        }
    }

    fn preview(&self, _context: skim::PreviewContext) -> skim::ItemPreview {
        match self {
            WorkflowAction::Run(workflow) => {
                let mut lines = vec![]; //format!("@{}/{}", self.namespace, self.name)];

                if let Some(description) = workflow.description.as_deref() {
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
            WorkflowAction::Create => skim::ItemPreview::AnsiText("".to_string()),
        }
    }

    fn output(&self) -> std::borrow::Cow<str> {
        match self {
            WorkflowAction::Run(workflow) => workflow.name.clone().into(),
            WorkflowAction::Create => "".into(),
        }
    }

    fn get_matching_ranges(&self) -> Option<&[(usize, usize)]> {
        None
    }
}

// Chay makes very large structs, Grant can't handle large structs
#[allow(clippy::large_enum_variant)]
enum WorkflowComponent {
    CheckBox {
        name: String,
        display_name: String,
        inner: CheckBox,
        value_if_true: String,
        value_if_false: String,
    },
    TextField {
        name: String,
        display_name: String,
        inner: TextField,
    },
    Picker {
        name: String,
        display_name: String,
        inner: Select,
    },
}

pub async fn execute(args: Vec<String>) -> Result<()> {
    // Parse args
    let name = args.first().map(String::from);

    let mut arg_pairs: HashMap<String, Value> = HashMap::new();
    let mut args = args.into_iter().skip(1);
    let mut arg = None;
    loop {
        arg = match arg {
            Some(arg) => match args.next() {
                Some(value) => match value.strip_prefix("--") {
                    Some(value) => {
                        arg_pairs.insert(arg, Value::Bool(true));
                        Some(value.to_string())
                    },
                    None => {
                        arg_pairs.insert(arg, Value::String(value));
                        None
                    },
                },
                None => {
                    arg_pairs.insert(arg, Value::Bool(true));
                    break;
                },
            },
            None => match args.next() {
                Some(new_arg) => match new_arg.strip_prefix("--") {
                    Some(new_arg) => Some(new_arg.to_string()),
                    None => bail!("Unexpected argument: {new_arg}"),
                },
                None => break,
            },
        }
    }
    let args = arg_pairs;

    let execution_method = match name.is_some() {
        true => "invoke",
        false => "search",
    };

    // Get workflow
    let workflow = match &name {
        Some(name) => {
            let (namespace, name) = match name.strip_prefix('@') {
                Some(name) => match name.split('/').collect::<Vec<&str>>()[..] {
                    [namespace, name] => (Some(namespace), name),
                    _ => bail!("Malformed workflow specifier, expects '@namespace/workflow-name': {name}",),
                },
                None => (None, name.as_ref()),
            };

            match request(
                Method::GET,
                format!("/workflows/{name}"),
                Some(&serde_json::json!({
                    "namespace": namespace,
                })),
                true,
            )
            .await?
            {
                Some(workflow) => workflow,
                None => {
                    match namespace {
                        Some(namespace) => bail!("Workflow does not exist: @{namespace}/{name}"),
                        None => bail!("Workflow does not exist for user: {name}"),
                    };
                },
            }
        },
        None => {
            let track_search = tokio::task::spawn(async move {
                let a: [(&'static str, &'static str); 0] = []; // dumb
                fig_telemetry::emit_track(TrackEvent::Other("Workflow Search Viewed".into()), TrackSource::Cli, a)
                    .await
                    .ok();
            });

            let workflows: Vec<Workflow> = Vec::new();
            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    use skim::prelude::*;

                    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

                    if workflows.is_empty() {
                        tx.send(Arc::new(WorkflowAction::Create)).ok();
                    }

                    for workflow in workflows.iter().rev() {
                        tx.send(Arc::new(WorkflowAction::Run(workflow.clone()))).ok();
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
                            .tac(true)
                            .build()
                            .unwrap(),
                        Some(rx),
                    );

                    let workflow = match output {
                        Some(out) => {
                            if out.is_abort {
                                return Ok(());
                            }

                            match out.selected_items.iter()
                                .map(|selected_item|
                                    (**selected_item)
                                        .as_any()
                                        .downcast_ref::<WorkflowAction>()
                                        .unwrap()
                                        .to_owned()
                                )
                                .next() {
                                Some(workflow) => {
                                    match workflow {
                                        WorkflowAction::Run(workflow) => workflow.clone(),
                                        WorkflowAction::Create => {
                                            println!();
                                            launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;
                                            println!();
                                            return match open_ui_element(UiElement::MissionControl).await {
                                                Ok(()) => Ok(()),
                                                Err(err) => Err(err.context("Could not open fig")),
                                            };
                                        },
                                    }
                                }
                                None => return Ok(()),
                            }
                        },
                        None => return Ok(()),
                    };
                } else if #[cfg(windows)] {
                    let workflow_names: Vec<String> = workflows
                        .iter()
                        .map(|workflow| {
                            workflow.display_name.clone().unwrap_or_else(|| workflow.name.clone())
                        })
                        .collect();

                    let selection = dialoguer::FuzzySelect::with_theme(&crate::util::dialoguer_theme())
                        .items(&workflow_names)
                        .default(0)
                        .interact()
                        .unwrap();

                    let workflow = workflows.remove(selection);
                }
            };

            track_search.await?;
            workflow
        },
    };

    execute!(std::io::stdout(), cursor::Hide)?;
    let mut spinner = Spinner::new(Spinners::Dots, "Loading workflow...".to_owned());

    let workflow_name = format!("@{}/{}", &workflow.namespace, &workflow.name);
    if workflow.template_version > SUPPORTED_SCHEMA_VERSION {
        return Err(anyhow!(
            "Could not execute {workflow_name} since it requires features not available in this version of Fig.\n\
            Please update to the latest version by running {} and try again.",
            "fig update".magenta(),
        ));
    }

    let track_execution = tokio::task::spawn(async move {
        fig_telemetry::emit_track(TrackEvent::Other("Workflow Executed".into()), TrackSource::Cli, [
            ("workflow", workflow_name.as_ref()),
            ("execution_method", execution_method),
        ])
        .await
        .ok();
    });

    let mut components: Vec<WorkflowComponent> = vec![];
    let parameter_count = workflow.parameters.len();
    for parameter in workflow.parameters {
        let display_name = parameter.display_name.unwrap_or_else(|| parameter.name.clone());
        let name = parameter.name;

        components.push(match parameter.parameter_type {
            ParameterType::Checkbox {
                true_value_substitution,
                false_value_substitution,
            } => WorkflowComponent::CheckBox {
                inner: CheckBox::new(args.get(&name).and_then(|val| val.as_bool()).unwrap_or(false))
                    .with_text(parameter.description.unwrap_or_else(|| "Toggle".to_string())),
                name,
                display_name,
                value_if_true: true_value_substitution,
                value_if_false: false_value_substitution,
            },
            ParameterType::Text { placeholder } => WorkflowComponent::TextField {
                inner: match placeholder {
                    Some(hint) => TextField::new().with_hint(hint),
                    None => TextField::new(),
                }
                .with_text(args.get(&name).and_then(|name| name.as_str()).unwrap_or("")),
                name,
                display_name,
            },
            ParameterType::Selector {
                placeholder,
                suggestions,
                generators,
            } => {
                let mut options = vec![];
                if let Some(suggestions) = suggestions {
                    for suggestion in suggestions {
                        options.push(suggestion.clone());
                    }
                }
                if let Some(generators) = generators {
                    for generator in generators {
                        match generator {
                            Generator::Named { .. } => {
                                return Err(anyhow!("Named generators aren't supported in workflows yet"));
                            },
                            Generator::Script { script } => {
                                if let Ok(output) = Command::new("bash").arg("-c").arg(script).output() {
                                    for option in String::from_utf8_lossy(&output.stdout).split('\n') {
                                        if !option.is_empty() {
                                            options.push(option.to_owned());
                                        }
                                    }
                                }
                            },
                        }
                    }
                }

                let mut select = match placeholder {
                    Some(placeholder) => Select::new(options, true).with_hint(&placeholder),
                    None => Select::new(options, true).with_hint("Search..."),
                };

                if let Some(arg) = args.get(&name).and_then(|name| name.as_str()) {
                    select.text = arg.to_owned();
                }

                WorkflowComponent::Picker {
                    name: name.clone(),
                    display_name,
                    inner: select,
                }
            },
        });
    }

    let mut frames: Vec<Frame> = components
        .iter_mut()
        .map(|component| match component {
            WorkflowComponent::CheckBox {
                display_name, inner, ..
            } => Frame::new(inner as &mut dyn Component).with_title(display_name.to_owned()),
            WorkflowComponent::TextField {
                display_name, inner, ..
            } => Frame::new(inner as &mut dyn Component).with_title(display_name.to_owned()),
            WorkflowComponent::Picker {
                display_name, inner, ..
            } => Frame::new(inner as &mut dyn Component).with_title(display_name.to_owned()),
        })
        .collect();

    let thin_border = BorderStyle::Ascii {
        top_left: '┌',
        top: '─',
        top_right: '┐',
        left: '│',
        right: '│',
        bottom_left: '└',
        bottom: '─',
        bottom_right: '┘',
    };

    let thick_border = BorderStyle::Ascii {
        top_left: '┏',
        top: '━',
        top_right: '┓',
        left: '┃',
        right: '┃',
        bottom_left: '┗',
        bottom: '━',
        bottom_right: '┛',
    };

    let focus_style = tui::style! {
        border_left_color: Color::White;
        border_right_color: Color::White;
        border_top_color: Color::White;
        border_bottom_color: Color::White;
        border_style: thick_border;
    };

    let unfocused_style = tui::style! {
        border_left_width: 1;
        border_top_width: 1;
        border_bottom_width: 1;
        border_right_width: 1;
        border_left_color: Color::DarkGrey;
        border_right_color: Color::DarkGrey;
        border_top_color: Color::DarkGrey;
        border_bottom_color: Color::DarkGrey;
        border_style: thin_border;
    };

    let style_sheet = tui::style_sheet! {
        "*" => {
            border_left_color: Color::Grey;
            border_right_color: Color::Grey;
            border_top_color: Color::Grey;
            border_bottom_color: Color::Grey;
        },
        "*:focus" => focus_style,
        "frame" => unfocused_style,
        "frame.title" => {
            color: Color::DarkGrey;
            padding_left: 1;
            padding_right: 1;
            margin_left: 1;
        },
        "frame.title:focus" => {
            color: Color::White;
            padding_left: 1;
            padding_right: 1;
            margin_left: 1;
        },
        "frame:focus" => focus_style,
        "textfield" => {
            padding_left: 2;
            color: Color::Grey;
        },
        "textfield:focus" =>  {
            ..focus_style;
            color: Color::White;
        },
        "disclosure.summary:focus" => {
            color: Color::White;
        },
        "disclosure.summary" => {
            color: Color::Grey;
        },
        "picker.item" => {
            padding_left: 0;
            color: Color::DarkGrey;
        },
        "picker.item:focus" => {
            padding_left: 0;
            color: Color::White;
        },
        "picker.selected" => {
            margin_left: 0;
            background_color:Color::DarkGrey;
            color:Color::Grey;
        },
        "picker.selected:focus" => {
            margin_left: 0;
            background_color: Color::White;
            color: Color::DarkGrey;
        },
        "checkbox" => {
            margin_left: 1;
        }
    };

    let mut model: Vec<&mut dyn Component> = vec![];
    let mut name = Label::new(workflow.display_name.as_ref().unwrap_or(&workflow.name));
    let mut description = workflow
        .description
        .as_ref()
        .map(|description| Label::new(description).with_margin_bottom(1));
    match description {
        Some(ref mut description) => {
            model.push(&mut name as &mut dyn Component);
            model.push(description as &mut dyn Component);
        },
        None => model.push(&mut name as &mut dyn Component),
    };
    for frame in &mut frames {
        model.push(frame as &mut dyn Component);
    }

    spinner.stop_with_message(String::new());
    execute!(std::io::stdout(), cursor::Show)?;

    if parameter_count > 0
        && EventLoop::new()
            .with_style_sheet(&style_sheet)
            .run::<std::io::Error, _>(
                ControlFlow::Wait,
                DisplayMode::AlternateScreen,
                &mut Form::new(model).with_margin_top(1).with_margin_left(2),
            )?
            > 0
    {
        // TODO: Add telemetry
        return Ok(());
    }

    let mut args: HashMap<&str, (Value, String)> = HashMap::new();
    for component in &components {
        match component {
            WorkflowComponent::CheckBox {
                name,
                inner,
                value_if_true,
                value_if_false,
                ..
            } => {
                args.insert(name, match inner.checked {
                    true => (true.into(), value_if_true.clone()),
                    false => (false.into(), value_if_false.clone()),
                });
            },
            WorkflowComponent::TextField { name, inner, .. } => {
                if !inner.text.is_empty() {
                    args.insert(name, (inner.text.clone().into(), inner.text.clone()));
                }
            },
            WorkflowComponent::Picker { name, inner, .. } => {
                if !inner.text.is_empty() {
                    args.insert(name, (inner.text.clone().into(), inner.text.clone()));
                }
            },
        };
    }

    if args.len() != parameter_count {
        return Err(anyhow!("Missing execution args"));
    }

    let mut command = format!("fig run @{}/{}", workflow.namespace, workflow.name);
    for (arg, (val, _)) in &args {
        use std::fmt::Write;

        match val {
            Value::Bool(b) => {
                if *b {
                    write!(command, " --{arg}").ok();
                }
            },
            Value::String(s) => {
                write!(command, " --{arg} {}", escape(s.into())).ok();
            },
            other => {
                write!(command, " --{arg} {}", escape(other.to_string().into())).ok();
            },
        }
    }

    if parameter_count > 0 {
        println!("{} {command}", "Executing:".bold().magenta());
    }

    cfg_if! {
        if #[cfg(feature = "deno")] {
            let map = args.into_iter().map(|(key, (v, _))| (key, v)).collect();
            let execute = execute_js_workflow(&workflow.template, &map);
        } else {
            let map = args.into_iter().map(|(key, (_, s))| (key, s)).collect();
            let execute = execute_bash_workflow(&workflow.name, &workflow.namespace, &workflow.tree, &map);
        }
    }

    // TODO:
    tokio::join! {
        execute,
        track_execution,
    }
    .0?;

    Ok(())
}

async fn execute_bash_workflow(
    name: &str,
    namespace: &str,
    tree: &[TreeElement],
    args: &HashMap<&str, String>,
) -> Result<()> {
    let start_time = time::OffsetDateTime::now_utc();
    let mut command = Command::new("bash");
    command.arg("-c");
    command.arg(tree.iter().fold(String::new(), |mut acc, branch| {
        match branch {
            TreeElement::String(string) => acc.push_str(string.as_str()),
            TreeElement::Token { name } => acc.push_str(&args[name.as_str()]),
        }
        acc
    }));

    let output = command.status()?;
    // std::io::stdout().write_all(&output.stdout)?;
    // std::io::stdout().write_all(&output.stderr)?;

    // let command_stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.code();
    if let Ok(execution_start_time) = start_time.format(&Rfc3339) {
        if let Ok(execution_duration) = i64::try_from((OffsetDateTime::now_utc() - start_time).whole_nanoseconds()) {
            request::<serde_json::Value, _, _>(
                Method::POST,
                format!("/workflows/{}/invocations", name),
                Some(&serde_json::json!({
                    "namespace": namespace,
                    "commandStderr": Value::Null,
                    "exitCode": exit_code,
                    "executionStartTime": execution_start_time,
                    "executionDuration": execution_duration
                })),
                true,
            )
            .await?;
        }
    }

    Ok(())
}

#[cfg(feature = "deno")]
pub async fn execute_js_workflow(script: &str, args: &HashMap<&str, Value>) -> Result<()> {
    use std::rc::Rc;
    use std::sync::Arc;

    use deno_core::error::AnyError;
    use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
    use deno_runtime::deno_web::BlobStore;
    use deno_runtime::permissions::Permissions;
    use deno_runtime::worker::{
        MainWorker,
        WorkerOptions,
    };
    use deno_runtime::{
        colors,
        BootstrapOptions,
    };

    fn get_error_class_name(e: &AnyError) -> &'static str {
        deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
    }

    let module_loader = Rc::new(deno_core::FsModuleLoader);
    let create_web_worker_cb = Arc::new(|_| {
        todo!("Web workers are not supported in the example");
    });
    let web_worker_preload_module_cb = Arc::new(|_| {
        todo!("Web workers are not supported in the example");
    });

    let options = WorkerOptions {
        bootstrap: BootstrapOptions {
            args: vec![],
            cpu_count: std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1),
            debug_flag: false,
            enable_testing_features: false,
            location: None,
            no_color: !colors::use_color(),
            is_tty: colors::is_tty(),
            runtime_version: "x".to_string(),
            ts_version: "x".to_string(),
            unstable: false,
            user_agent: "fig-cli/workflow".to_string(),
        },
        extensions: vec![],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        seed: None,
        source_map_getter: None,
        format_js_error_fn: None,
        web_worker_preload_module_cb,
        create_web_worker_cb,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader,
        get_error_class_fn: Some(&get_error_class_name),
        origin_storage_dir: None,
        blob_store: BlobStore::default(),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
    };

    let permissions = Permissions::allow_all();

    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("script.js");

    tokio::fs::write(&file, script).await.unwrap();

    let specificer = deno_core::ModuleSpecifier::from_file_path(file).unwrap();

    let mut worker = MainWorker::bootstrap_from_options(specificer.clone(), permissions, options);

    worker.execute_script("[fig-init]", "const args = {}").unwrap();

    for (key, value) in args {
        worker
            .execute_script("[fig-init]", &format!("args.{key}={value};const ${key}=args.{key};"))
            .unwrap();
    }

    worker.execute_main_module(&specificer).await.unwrap();

    worker.run_event_loop(false).await.unwrap();

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
