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

use cfg_if::cfg_if;
use clap::Args;
use crossterm::style::Stylize;
use eyre::{
    bail,
    eyre,
    Result,
};
use fig_api_client::workflows::{
    workflows,
    FileType,
    Generator,
    Parameter,
    ParameterType,
    Predicate,
    Rule,
    RuleType,
    TreeElement,
    Workflow,
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
use serde_json::Value;
#[cfg(unix)]
use skim::SkimItem;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tokio::io::AsyncWriteExt;
use tracing::warn;
use tui::{
    BorderStyle,
    CheckBox,
    Color,
    Component,
    Container,
    ControlFlow,
    DisplayMode,
    EventLoop,
    FilePicker,
    InputMethod,
    Label,
    Paragraph,
    Select,
    TextField,
};

const SUPPORTED_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Args)]
pub struct WorkflowArgs {
    // Flags can be added here
    #[arg(allow_hyphen_values = true)]
    args: Vec<String>,
}

impl WorkflowArgs {
    pub async fn execute(self) -> Result<()> {
        execute(self.args).await
    }
}

#[cfg(unix)]
enum WorkflowAction {
    Run(Box<fig_api_client::workflows::Workflow>),
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

async fn write_workflows() -> Result<(), eyre::Report> {
    for workflow in workflows(SUPPORTED_SCHEMA_VERSION).await? {
        let mut file = tokio::fs::File::create(
            directories::workflows_cache_dir()?.join(format!("{}.{}.json", workflow.namespace, workflow.name)),
        )
        .await?;
        file.write_all(serde_json::to_string_pretty(&workflow)?.as_bytes())
            .await?;
    }
    Ok(())
}

async fn get_workflows() -> Result<Vec<Workflow>> {
    let mut workflows = vec![];
    for file in directories::workflows_cache_dir()?.read_dir()?.flatten() {
        if let Some(name) = file.file_name().to_str() {
            if name.ends_with(".json") {
                workflows.push(serde_json::from_slice::<Workflow>(
                    &tokio::fs::read(file.path()).await?,
                )?);
            }
        }
    }
    Ok(workflows)
}

pub async fn execute(env_args: Vec<String>) -> Result<()> {
    // Create cache dir
    tokio::fs::create_dir_all(directories::workflows_cache_dir()?).await?;

    let workflows = get_workflows().await?;

    // Must come after we get workflows
    let mut write_workflows: Option<tokio::task::JoinHandle<Result<(), _>>> = Some(tokio::spawn(write_workflows()));

    // Parse args
    let workflow_name = env_args.first().map(String::from);
    let (execution_method, workflow) = match workflow_name {
        Some(name) => {
            let (namespace, name) = match name.strip_prefix('@') {
                Some(name) => match name.split('/').collect::<Vec<&str>>()[..] {
                    [namespace, name] => (Some(namespace), name),
                    _ => bail!("Malformed workflow specifier, expects '@namespace/workflow-name': {name}",),
                },
                None => (None, name.as_ref()),
            };

            let workflow = match namespace {
                Some(namespace) => workflows
                    .into_iter()
                    .find(|workflow| workflow.name == name && workflow.namespace == namespace),
                None => workflows
                    .into_iter()
                    .find(|workflow| workflow.name == name && workflow.is_owned_by_user),
            };

            let workflow = match workflow {
                Some(workflow) => workflow,
                None => {
                    write_workflows.take().unwrap().await??;

                    let workflows = get_workflows().await?;

                    let workflow = match namespace {
                        Some(namespace) => workflows
                            .into_iter()
                            .find(|workflow| workflow.name == name && workflow.namespace == namespace),
                        None => workflows
                            .into_iter()
                            .find(|workflow| workflow.name == name && workflow.is_owned_by_user),
                    };

                    if workflow.is_none() {
                        eprintln!("Workflow not found");
                        return Ok(());
                    }

                    workflow.unwrap()
                },
            };

            ("invoke", workflow)
        },
        None => {
            fig_telemetry::dispatch_emit_track(
                TrackEvent::new(
                    TrackEventType::WorkflowSearchViewed,
                    TrackSource::Cli,
                    env!("CARGO_PKG_VERSION").into(),
                    empty::<(&str, &str)>(),
                ),
                false,
            )
            .await
            .ok();

            if let Err(err) = write_workflows.take().unwrap().await? {
                eprintln!("Could not load remote workflows!\nFalling back to local cache.");
                warn!("Failed to acquire remote workflow definitions: {err}");
            }

            let mut workflows = get_workflows().await?;
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
                                        WorkflowAction::Run(workflow) => ("search", *workflow.clone()),
                                        WorkflowAction::Create => {
                                            fig_util::launch_fig(true, true)?;
                                            return match open_ui_element(UiElement::MissionControl, Some("/workflows".to_string())).await {
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

                    ("search", workflows.remove(selection))
                }
            }
        },
    };

    let workflow_name = format!("@{}/{}", &workflow.namespace, &workflow.name);
    if workflow.template_version > SUPPORTED_SCHEMA_VERSION {
        return Err(eyre!(
            "Could not execute {workflow_name} since it requires features not available in this version of Fig.\n\
            Please update to the latest version by running {} and try again.",
            "fig update".magenta(),
        ));
    }

    if let Some(ruleset) = &workflow.rules {
        if !rules_met(ruleset)? {
            return Ok(());
        }
    }

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

    let map = parse_args(args.borrow(), &workflow.parameters);
    if let Ok(map) = map {
        if execution_method == "invoke" {
            execute_bash_workflow(&workflow.name, &workflow.namespace, &workflow.tree, &map).await?;
        } else if execution_method == "search" {
            let args = &args.take();
            if send_figterm(map_args_to_command(&workflow, args), true).await.is_err() {
                execute_bash_workflow(&workflow.name, &workflow.namespace, &workflow.tree, args).await?;
            }
        }
        return Ok(());
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
            color: Color::DarkYellow;
            border_left_color: Color::DarkYellow;
            border_right_color: Color::DarkYellow;
            border_top_color: Color::DarkYellow;
            border_bottom_color: Color::DarkYellow;
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
            width: Some(100);
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
            width: Some(98);
            padding_left: 1;
            padding_right: 2;
        }
    };

    if !workflow.parameters.is_empty() {
        let mut preview = false;
        let mut event_loop = EventLoop::new(DisplayMode::AlternateScreen)?;

        loop {
            let mut header = Paragraph::new();
            header.push_line_break();
            header.push_styled_text(
                workflow.display_name.as_ref().unwrap_or(&workflow.name),
                None,
                None,
                true,
            );
            header.push_styled_text(
                format!(" • @{}", workflow.namespace),
                Some(Color::DarkGrey),
                None,
                false,
            );

            if let Some(description) = &workflow.description {
                if !description.is_empty() {
                    header.push_line_break();
                    header.push_styled_text(description, Some(Color::DarkYellow), None, false);
                }
            }

            let mut header = Component::from(header);
            header.style.with_margin_bottom(1);

            let mut components = vec![header];

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
                                                    return Err(eyre!(
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
                                ParameterType::Text { placeholder } => Component::from(
                                    TextField::new(move |signal| {
                                        args.borrow_mut().insert(parameter_name.clone(), signal);
                                    })
                                    .with_text(parameter_value)
                                    .with_hint(placeholder.to_owned().unwrap_or_default()),
                                ),
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

                                    if !checked {
                                        args.borrow_mut().insert(parameter_name.clone(), false_value.clone());
                                    }

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
                                ParameterType::Path { file_type, extensions } => {
                                    let (files, folders) = match file_type {
                                        FileType::Any => (true, true),
                                        FileType::FileOnly => (true, false),
                                        FileType::FolderOnly => (false, true),
                                    };

                                    Component::from(FilePicker::new(
                                        std::env::current_dir()?,
                                        files,
                                        folders,
                                        extensions.clone(),
                                        move |signal| {
                                            args.borrow_mut()
                                                .insert(parameter_name.clone(), signal.to_string_lossy().to_string());
                                        },
                                    ))
                                },
                            },
                        ])));
                    }
                },
            };

            let mut keybindings = Paragraph::new();

            keybindings.push_styled_text("enter", Some(Color::DarkYellow), None, false);
            keybindings.push_styled_text(" select • ", Some(Color::Reset), None, false);
            keybindings.push_styled_text("tab", Some(Color::DarkYellow), None, false);
            keybindings.push_styled_text(" next • ", Some(Color::Reset), None, false);
            keybindings.push_styled_text("shift+tab", Some(Color::DarkYellow), None, false);
            keybindings.push_styled_text(" previous • ", Some(Color::Reset), None, false);
            keybindings.push_styled_text("⎵", Some(Color::DarkYellow), None, false);
            keybindings.push_styled_text(" toggle • ", Some(Color::Reset), None, false);
            keybindings.push_styled_text("⌃o", Some(Color::DarkYellow), None, false);
            keybindings.push_styled_text(" preview", Some(Color::Reset), None, false);

            let mut keybindings = Component::from(keybindings);
            keybindings.style.with_margin_left(0).with_width(Some(110));
            components.push(keybindings);

            let mut view = Component::from(Container::new(components));
            view.style
                .with_border_style(BorderStyle::None)
                .with_padding_top(0)
                .with_margin_left(2)
                .with_margin_right(2);

            match event_loop.run(&mut view, &input_method, Some(&style_sheet), ControlFlow::Wait)? {
                ControlFlow::Exit(0) => break,
                ControlFlow::Exit(_) => {
                    fig_telemetry::dispatch_emit_track(
                        TrackEvent::new(
                            TrackEventType::WorkflowCancelled,
                            TrackSource::Cli,
                            env!("CARGO_PKG_VERSION").into(),
                            [
                                ("workflow", workflow_name.as_ref()),
                                ("execution_method", execution_method),
                            ],
                        ),
                        false,
                    )
                    .await
                    .ok();
                    return Ok(());
                },
                ControlFlow::Reenter(_) => preview = !preview,
                _ => (),
            }
        }
    }

    let command = map_args_to_command(&workflow, &args.borrow());

    fig_telemetry::dispatch_emit_track(
        TrackEvent::new(
            TrackEventType::WorkflowExecuted,
            TrackSource::Cli,
            env!("CARGO_PKG_VERSION").into(),
            [
                ("workflow", workflow_name.as_ref()),
                ("execution_method", execution_method),
            ],
        ),
        false,
    )
    .await
    .ok();

    if let Some(task) = write_workflows {
        if let Err(err) = task.await? {
            eprintln!("Failed to update workflows from remote: {err}");
        }
    }

    cfg_if! {
        if #[cfg(feature = "deno")] {
            let map = args.into_iter().map(|(key, (v, _))| (key, v)).collect();
            execute_js_workflow(&workflow.template, &map)?;
        } else {
            if send_figterm(command, true).await.is_err() {
                execute_bash_workflow(&workflow.name, &workflow.namespace, &workflow.tree, &args.take()).await?;
            }
        }
    }

    Ok(())
}

fn parse_args(
    args: core::cell::Ref<HashMap<String, String>>,
    parameters: &Vec<Parameter>,
) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for parameter in parameters {
        let value = args.get(&parameter.name);
        map.insert(parameter.name.clone(), match value {
            Some(value) => match &parameter.parameter_type {
                ParameterType::Checkbox {
                    true_value_substitution,
                    false_value_substitution,
                } => match value {
                    value if value == "true" => true_value_substitution.to_owned(),
                    _ => false_value_substitution.to_owned(),
                },
                _ => value.to_owned(),
            },
            None => return Err(eyre!("Missing execution args")),
        });
    }

    Ok(map)
}

fn map_args_to_command(workflow: &Workflow, args: &HashMap<String, String>) -> String {
    let mut command = format!("fig run {}", match workflow.is_owned_by_user {
        true => workflow.name.clone(),
        false => format!("@{}/{}", &workflow.namespace, &workflow.name),
    });
    for (arg, val) in args {
        use std::fmt::Write;

        match val {
            val if val == "true" => write!(command, " --{arg}").ok(),
            val => write!(command, " --{arg} {}", escape(val.into())).ok(),
        };
    }

    command
}

async fn send_figterm(text: String, execute: bool) -> eyre::Result<()> {
    let session_id = std::env::var("TERM_SESSION_ID")?;
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

async fn execute_bash_workflow(
    name: &str,
    namespace: &str,
    tree: &[TreeElement],
    args: &HashMap<String, String>,
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

    let output = command.status();

    let exit_code = output.ok().and_then(|output| output.code());
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
                .await
                .ok();
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

    let specifier = deno_core::ModuleSpecifier::from_file_path(file).unwrap();

    let mut worker = MainWorker::bootstrap_from_options(specifier.clone(), permissions, options);

    worker.execute_script("[fig-init]", "const args = {}").unwrap();

    for (key, value) in args {
        worker
            .execute_script("[fig-init]", &format!("args.{key}={value};const ${key}=args.{key};"))
            .unwrap();
    }

    worker.execute_main_module(&specifier).await.unwrap();

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

#[cfg(test)]
mod test {
    use fig_api_client::workflows::Workflow;

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

        let workflow = serde_json::from_value::<Workflow>(json)?;
        assert!(workflow.rules.is_some());

        let ruleset = workflow.rules.unwrap();
        rules_met(&ruleset)?;

        Ok(())
    }
}
