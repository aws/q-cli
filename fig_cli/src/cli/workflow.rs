use std::collections::HashMap;
use std::process::Command;

use anyhow::{
    anyhow,
    bail,
    Result,
};
use crossterm::style::Stylize;
use fig_telemetry::{
    TrackEvent,
    TrackSource,
};
use reqwest::Method;
use serde::{
    Deserialize,
    Serialize,
};
use skim::SkimItem;
use tui::components::{
    CheckBox,
    CollapsiblePicker,
    FilterablePicker,
    Frame,
    Label,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Workflow {
    name: String,
    namespace: String,
    display_name: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    template: String,
    parameters: Vec<Parameter>,
    tree: Vec<TreeElement>,
}

// impl AsRef<str> for Workflow {
//    fn as_ref(&self) -> &str {
//        self.name.as_str()
//    }
//}

impl SkimItem for Workflow {
    fn text(&self) -> std::borrow::Cow<str> {
        let tags = match &self.tags {
            Some(tags) => tags.join(" "),
            None => String::new(),
        };

        format!(
            "{} {} @{}/{} {}",
            self.display_name.as_deref().unwrap_or_default(),
            self.name,
            self.namespace,
            self.name,
            tags
        )
        .into()
    }

    fn display<'a>(&'a self, context: skim::DisplayContext<'a>) -> skim::AnsiString<'a> {
        let tags = match &self.tags {
            Some(tags) if !tags.is_empty() => format!(" |{}| ", tags.join("|")),
            _ => String::new(),
        };
        let tag_len = tags.len();
        let namespace_name = format!("@{}/{}", self.namespace, self.name);
        skim::AnsiString::parse(
            match &self.display_name {
                None => format!(
                    "{}{}{}{}",
                    self.name.clone().bold(),
                    tags.dark_grey(),
                    " ".repeat(context.container_width - self.name.len() - tag_len - namespace_name.len() - 1),
                    namespace_name.dark_grey()
                ),
                Some(display_name) => {
                    format!(
                        "{}{}{}{}",
                        display_name.clone().bold(),
                        tags.dark_grey(),
                        " ".repeat(context.container_width - display_name.len() - tag_len - namespace_name.len() - 1),
                        namespace_name.dark_grey()
                    )
                },
            }
            .as_str(),
        )
    }

    fn preview(&self, context: skim::PreviewContext) -> skim::ItemPreview {
        let mut lines = vec![format!("@{}/{}", self.namespace, self.name)];

        if let Some(description) = self.description.as_deref() {
            if !description.is_empty() {
                lines.push(description.to_owned());
            }
        }

        lines.push("━".repeat(context.width).black().to_string());
        lines.push(self.template.clone());

        skim::ItemPreview::AnsiText(lines.join("\n"))
    }

    fn output(&self) -> std::borrow::Cow<str> {
        self.name.clone().into()
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
        inner: CollapsiblePicker<FilterablePicker>,
    },
}

pub async fn execute(args: Vec<String>) -> Result<()> {
    // Parse args
    let name = args.first().map(String::from);

    let mut arg_pairs: HashMap<String, String> = HashMap::new();
    let mut args = args.into_iter().skip(1);
    let mut arg = None;
    loop {
        arg = match arg {
            Some(arg) => match args.next() {
                Some(value) => match value.strip_prefix("--") {
                    Some(value) => {
                        arg_pairs.insert(arg, "true".to_string());
                        Some(value.to_string())
                    },
                    None => {
                        arg_pairs.insert(arg, value);
                        None
                    },
                },
                None => {
                    arg_pairs.insert(arg, "true".to_string());
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
            let workflows: Vec<Workflow> = request(Method::GET, "/workflows", None, true).await?;
            let track_search = tokio::task::spawn(async move {
                let a: [(&'static str, &'static str); 0] = []; // dumb
                fig_telemetry::emit_track(TrackEvent::Other("Workflow Search Viewed".into()), TrackSource::Cli, a)
                    .await
                    .ok();
            });

            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    use skim::prelude::*;

                    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
                    for workflow in workflows.iter() {
                        tx.send(Arc::new(workflow.clone())).ok();
                    }
                    drop(tx);

                    let output = Skim::run_with(
                        &SkimOptionsBuilder::default()
                            .height(Some("50%"))
                            // .preview(Some(""))
                            // .preview_window(Some("down"))
                            .reverse(true)
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
                                        .downcast_ref::<Workflow>()
                                        .unwrap()
                                        .to_owned()
                                )
                                .next() {
                                Some(workflow) => workflow,
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

    let track_execution = tokio::task::spawn(async move {
        fig_telemetry::emit_track(TrackEvent::Other("Workflow Executed".into()), TrackSource::Cli, [(
            "execution_method",
            execution_method,
        )])
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
                inner: CheckBox::new(args.get(&name).is_some())
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
                .with_text(args.get(&name).unwrap_or(&String::new())),
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
                            Generator::Named { .. } => todo!(),
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

                let mut index = 0;
                if let Some(arg) = args.get(&name) {
                    for (i, option) in options.iter().enumerate() {
                        if option == arg {
                            index = i;
                            break;
                        }
                    }
                };

                WorkflowComponent::Picker {
                    name: name.clone(),
                    display_name,
                    inner: match placeholder {
                        Some(placeholder) => CollapsiblePicker::new(options).with_placeholder(&placeholder),
                        None => CollapsiblePicker::new(options),
                    }
                    .with_index(index),
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
        None => {
            name = name.with_margin_bottom(1);
            model.push(&mut name as &mut dyn Component);
        },
    };
    for frame in &mut frames {
        model.push(frame as &mut dyn Component);
    }

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

    let mut args: HashMap<&str, &str> = HashMap::new();
    for component in &components {
        match component {
            WorkflowComponent::CheckBox {
                name,
                inner,
                value_if_true,
                value_if_false,
                ..
            } => args.insert(name, match inner.checked {
                true => value_if_true,
                false => value_if_false,
            }),
            WorkflowComponent::TextField { name, inner, .. } => args.insert(name, &inner.text),
            WorkflowComponent::Picker { name, inner, .. } => args.insert(name, match inner.selected_item() {
                Some(selected) => selected,
                None => return Err(anyhow!("Missing entry for field: {name}")),
            }),
        };
    }

    if args.len() != parameter_count {
        return Err(anyhow!("Missing execution args"));
    }

    let mut command = format!("fig run @{}/{}", workflow.namespace, workflow.name);
    for (arg, val) in &args {
        command.push_str(&format!(" --{arg} \"{}\"", val.escape_default()));
    }

    println!("{} {command}", "Executing:".bold().magenta());
    // TODO:
    tokio::join! {
        execute_workflow(workflow.tree, args),
        track_execution,
    }
    .0?;

    Ok(())
}

async fn execute_workflow(tree: Vec<TreeElement>, args: HashMap<&str, &str>) -> Result<()> {
    let mut command = Command::new("bash");
    command.arg("-c");
    command.arg(tree.into_iter().fold(String::new(), |mut acc, branch| {
        match branch {
            TreeElement::String(string) => acc.push_str(string.as_str()),
            TreeElement::Token { name } => acc.push_str(args[name.as_str()]),
        }

        acc
    }));
    command.status()?;

    Ok(())
}
