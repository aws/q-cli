use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::Ordering as StdOrdering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};
use std::iter::empty;
use std::process::Command;
use std::rc::Rc;

use anyhow::{
    anyhow,
    bail,
    Result,
};
use clap::Args;
use crossterm::style::Stylize;
use crossterm::{
    cursor,
    execute,
};
#[cfg(unix)]
use fig_ipc::command::open_ui_element;
#[cfg(unix)]
use fig_proto::local::UiElement;
use fig_request::Request;
use fig_telemetry::{
    TrackEvent,
    TrackSource,
};
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
use tui::{
    BorderStyle,
    CheckBox,
    Color,
    Component,
    Container,
    ControlFlow,
    DisplayMode,
    EventLoop,
    InputMethod,
    Label,
    Paragraph,
    Select,
    TextField,
};

#[cfg(unix)]
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
    depends_on: Vec<String>,
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
    last_invoked_at: Option<String>,
    tags: Option<Vec<String>>,
    parameters: Vec<Parameter>,
    namespace: String,
    template: String,
    tree: Vec<TreeElement>,
    is_owned_by_user: Option<bool>,
}

#[cfg(unix)]
enum WorkflowAction {
    Run(Box<Workflow>),
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

pub async fn execute(env_args: Vec<String>) -> Result<()> {
    // Get workflows early
    let mut workflows: Vec<Workflow> = Request::get("/workflows").auth().deser_json().await?;

    // Parse args
    let workflow_name = env_args.first().map(String::from);
    let execution_method = match workflow_name {
        Some(_) => "invoke",
        None => "search",
    };

    // Get matching workflows
    if let Some(workflow_name) = workflow_name {
        let (namespace, name) = match workflow_name.strip_prefix('@') {
            Some(name) => match name.split('/').collect::<Vec<&str>>()[..] {
                [namespace, name] => (Some(namespace), name),
                _ => bail!("Malformed workflow specifier, expects '@namespace/workflow-name': {name}",),
            },
            None => (None, workflow_name.as_ref()),
        };

        workflows = workflows
            .into_iter()
            .filter(|c| {
                c.name == name
                    && match namespace {
                        Some(namespace) => c.namespace == namespace,
                        None => true,
                    }
            })
            .collect();

        if workflows.is_empty() {
            bail!("No matching workflows for {workflow_name}");
        }
    };

    let workflow = match workflows.len() {
        1 => workflows.pop().unwrap(),
        _ => {
            fig_telemetry::dispatch_emit_track(
                TrackEvent::WorkflowSearchViewed,
                TrackSource::Cli,
                empty::<(&str, &str)>(),
            )
            .await
            .ok();

            workflows.sort_by(|a, b| match (&a.last_invoked_at, &b.last_invoked_at) {
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
                    use skim::prelude::*;

                    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

                    if workflows.is_empty() {
                        tx.send(Arc::new(WorkflowAction::Create)).ok();
                    }

                    for workflow in workflows.iter().rev() {
                        tx.send(Arc::new(WorkflowAction::Run(Box::new(workflow.clone())))).ok();
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
                                        .downcast_ref::<WorkflowAction>()
                                        .unwrap()
                                        .to_owned()
                                )
                                .next() {
                                Some(workflow) => {
                                    match workflow {
                                        WorkflowAction::Run(workflow) => *workflow.clone(),
                                        WorkflowAction::Create => {
                                            println!();
                                            launch_fig(LaunchOptions::new().wait_for_activation().verbose())?;
                                            println!();
                                            return match open_ui_element(UiElement::MissionControl, Some("/workflows".to_string())).await {
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
                    }
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

                    workflows.remove(selection)
                }
            }
        },
    };

    let mut env_args = env_args.into_iter().skip(1);
    let args: Rc<RefCell<HashMap<String, String>>> = Rc::new(RefCell::new(HashMap::new()));
    let mut arg = None;
    loop {
        arg = match arg {
            Some(arg) => match env_args.next() {
                Some(value) => match value.strip_prefix("--") {
                    Some(value) => {
                        if let Some(parameter) = workflow.parameters.iter().find(|p| p.name == arg) {
                            match &parameter.parameter_type {
                                ParameterType::Checkbox {
                                    true_value_substitution,
                                    ..
                                } => args.borrow_mut().insert(arg, true_value_substitution.clone()),
                                _ => args.borrow_mut().insert(arg, "true".to_string()),
                            };
                        }
                        Some(value.to_string())
                    },
                    None => {
                        args.borrow_mut().insert(arg, value);
                        None
                    },
                },
                None => {
                    if let Some(parameter) = workflow.parameters.iter().find(|p| p.name == arg) {
                        match &parameter.parameter_type {
                            ParameterType::Checkbox {
                                true_value_substitution,
                                ..
                            } => args.borrow_mut().insert(arg, true_value_substitution.clone()),
                            _ => args.borrow_mut().insert(arg, "true".to_string()),
                        };
                    }
                    break;
                },
            },
            None => match env_args.next() {
                Some(new_arg) => match new_arg.strip_prefix("--") {
                    Some(new_arg) => Some(new_arg.to_string()),
                    None => bail!("Unexpected argument: {new_arg}"),
                },
                None => break,
            },
        }
    }

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

    let style_sheet = tui::style_sheet! {
        "*" => {
            border_left_color: Color::DarkGrey;
            border_right_color: Color::DarkGrey;
            border_top_color: Color::DarkGrey;
            border_bottom_color: Color::DarkGrey;
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
            color: Color::White;
            border_left_color: Color::White;
            border_right_color: Color::White;
            border_top_color: Color::White;
            border_bottom_color: Color::White;
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
            padding_left: 1;
            padding_right: 1;
        },
        "div" => {
            color: Color::DarkGrey;
            width: 110;
            padding_top: -1;
            border_left_width: 1;
            border_top_width: 1;
            border_bottom_width: 1;
            border_right_width: 1;
        },
        "h1" => {
            margin_left: 1;
            padding_left: 1;
            padding_right: 1;
        },
        "p" => {
            padding_left: 1;
            padding_right: 1;
        },
        "select" => {
            padding_left: 1;
            padding_right: 1;
        },
        "input:text" => {
            width: 108;
            padding_left: 1;
            padding_right: 2;
        }
    };

    spinner.stop_with_message(String::new());
    execute!(std::io::stdout(), cursor::Show)?;

    if !workflow.parameters.is_empty() {
        let mut preview = false;
        let mut event_loop = EventLoop::new(DisplayMode::AlternateScreen)?;

        loop {
            let mut header = Paragraph::new();
            header.push_styled_text(
                workflow.display_name.as_ref().unwrap_or(&workflow.name),
                None,
                None,
                true,
            );
            header.push_styled_text(format!(" | {}", workflow.namespace), Some(Color::DarkGrey), None, false);
            header.push_line_break();
            if let Some(description) = &workflow.description {
                header.push_text(description);
            }

            let mut components = vec![Component::from(header).with_margin_bottom(1)];

            let input_method;
            match preview {
                true => {
                    input_method = InputMethod::None;

                    let colors = [Color::Magenta, Color::Blue, Color::Cyan];
                    let mut paragraph = Paragraph::new();
                    for element in &workflow.tree {
                        match element {
                            TreeElement::String(s) => paragraph.push_text(s),
                            TreeElement::Token { name } => {
                                let mut hasher = DefaultHasher::new();
                                name.hash(&mut hasher);
                                let hash = hasher.finish() as usize;

                                paragraph.push_styled_text(
                                    match args.borrow().get(name.as_str()) {
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

                    components.push(Component::from(Container::new(vec![
                        Component::from(Label::new("Preview", false)),
                        Component::from(paragraph),
                    ])));
                },
                false => {
                    input_method = InputMethod::Form;

                    for parameter in &workflow.parameters {
                        let args = args.clone();
                        let parameter_name = parameter.name.clone();
                        let parameter_value = args.borrow().get(&parameter_name).unwrap_or(&String::default()).clone();

                        components.push(Component::from(Container::new(vec![
                            Component::from(Label::new(
                                parameter.display_name.clone().unwrap_or_else(|| parameter.name.clone()),
                                false,
                            )),
                            match &parameter.parameter_type {
                                ParameterType::Checkbox {
                                    true_value_substitution,
                                    false_value_substitution,
                                } => {
                                    let true_value = true_value_substitution.clone();
                                    let false_value = false_value_substitution.clone();
                                    let checked = args
                                        .borrow_mut()
                                        .get(&parameter_name)
                                        .map(|c| c == &true_value)
                                        .unwrap_or(false);
                                    Component::from(CheckBox::new(
                                        parameter.description.to_owned().unwrap_or_else(|| "Toggle".to_string()),
                                        checked,
                                        move |signal| {
                                            args.borrow_mut().insert(parameter_name.clone(), match signal {
                                                true => true_value.clone(),
                                                false => false_value.clone(),
                                            });
                                        },
                                    ))
                                },
                                ParameterType::Text { placeholder } => Component::from(
                                    TextField::new(move |signal| {
                                        args.borrow_mut().insert(parameter_name.clone(), signal);
                                    })
                                    .with_text(parameter_value)
                                    .with_hint(placeholder.to_owned().unwrap_or_else(|| "".to_string())),
                                ),
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
                                                    return Err(anyhow!(
                                                        "Named generators aren't supported in workflows yet"
                                                    ));
                                                },
                                                Generator::Script { script } => {
                                                    if let Ok(output) =
                                                        Command::new("bash").arg("-c").arg(script).output()
                                                    {
                                                        for suggestion in
                                                            String::from_utf8_lossy(&output.stdout).split('\n')
                                                        {
                                                            if !suggestion.is_empty() {
                                                                options.push(suggestion.to_owned());
                                                            }
                                                        }
                                                    }
                                                },
                                            }
                                        }
                                    }

                                    Component::from(
                                        Select::new(options, true, move |signal| {
                                            args.borrow_mut().insert(parameter_name.clone(), signal);
                                        })
                                        .with_text(parameter_value)
                                        .with_hint(placeholder.as_deref().unwrap_or("Search...")),
                                    )
                                },
                            },
                        ])));
                    }
                },
            };
            components.push(
                Component::from(Label::new(
                    "Preview: CTRL+O | Next: TAB | Prev: SHIFT+TAB | Select: SPACE | Execute: ENTER",
                    false,
                ))
                .with_color(Color::DarkGrey)
                .with_background_color(Color::White)
                .with_margin_left(0)
                .with_width(110),
            );

            let mut view = Component::from(Container::new(components))
                .with_border_style(BorderStyle::None)
                .with_padding_top(0)
                .with_margin_left(2)
                .with_margin_right(2);

            match event_loop.run(&mut view, &input_method, Some(&style_sheet), ControlFlow::Wait)? {
                ControlFlow::Exit(0) => break,
                ControlFlow::Exit(_) => {
                    fig_telemetry::dispatch_emit_track(TrackEvent::WorkflowCancelled, TrackSource::Cli, [
                        ("workflow", workflow_name.as_ref()),
                        ("execution_method", execution_method),
                    ])
                    .await
                    .ok();
                    return Ok(());
                },
                ControlFlow::Reenter(_) => preview = !preview,
                _ => (),
            }
        }
    }

    let mut command = format!("fig run {}", match workflow.is_owned_by_user.unwrap_or(false) {
        true => workflow.name.clone(),
        false => format!("@{}/{}", &workflow.namespace, &workflow.name),
    });
    for (arg, val) in &*args.borrow() {
        use std::fmt::Write;

        match val {
            val if val == "true" => write!(command, " --{arg}").ok(),
            val => write!(command, " --{arg} {}", escape(val.into())).ok(),
        };
    }

    if !workflow.parameters.is_empty() {
        println!("{} {command}", "Executing:".bold().magenta());
    }

    cfg_if! {
        if #[cfg(feature = "deno")] {
            let map = args.into_iter().map(|(key, (v, _))| (key, v)).collect();
            execute_js_workflow(&workflow.template, &map)?;
        } else {
            let mut map = HashMap::new();
            for parameter in &workflow.parameters {
                let args = args.borrow();
                let value = args.get(&parameter.name);
                map.insert(parameter.name.as_str(), match value {
                    Some(value) => match &parameter.parameter_type {
                        ParameterType::Checkbox { true_value_substitution, false_value_substitution } => match value {
                            value if value == "true" => true_value_substitution.to_owned(),
                            _ => false_value_substitution.to_owned(),
                        },
                        _ => value.to_owned(),
                    },
                    None => return Err(anyhow!("Missing execution args")),
                });
            }

            execute_bash_workflow(&workflow.name, &workflow.namespace, &workflow.tree, &map).await?;
        }
    }

    fig_telemetry::dispatch_emit_track(TrackEvent::WorkflowExecuted, TrackSource::Cli, [
        ("workflow", workflow_name.as_ref()),
        ("execution_method", execution_method),
    ])
    .await
    .ok();

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

    let exit_code = output.code();
    if let Ok(execution_start_time) = start_time.format(&Rfc3339) {
        if let Ok(execution_duration) = i64::try_from((OffsetDateTime::now_utc() - start_time).whole_nanoseconds()) {
            Request::post(format!("/workflows/{name}/invocations"))
                .body(serde_json::json!({
                    "namespace": namespace,
                    "commandStderr": Value::Null,
                    "exitCode": exit_code,
                    "executionStartTime": execution_start_time,
                    "executionDuration": execution_duration
                }))
                .auth()
                .send()
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
