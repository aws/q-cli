use std::collections::HashMap;
use std::process::Command;

use anyhow::{
    anyhow,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
enum Generator {
    #[serde(rename_all = "camelCase")]
    Named { name: String },
    #[serde(rename_all = "camelCase")]
    Script { script: String },
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Parameter {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    parameter_type: ParameterType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
enum TreeElement {
    String(String),
    Token { name: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Workflow {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    template: String,
    parameters: Vec<Parameter>,
    tree: Vec<TreeElement>,
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
    let name = args.get(1).map(String::from);
    let mut arg_pairs: HashMap<String, String> = HashMap::new();
    let mut args = args.into_iter().skip(2);
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
                    None => anyhow::bail!("Unexpected argument: {}", new_arg),
                },
                None => break,
            },
        }
    }
    let args = arg_pairs;

    // Get workflow
    let workflow = match &name {
        Some(name) => match request(Method::GET, format!("/workflows/{name}"), None, true).await? {
            Some(workflow) => workflow,
            None => return Err(anyhow!("Workflow does not exist with name: {}", name)),
        },
        None => {
            let mut workflows: Vec<Workflow> = request(Method::GET, "/workflows", None, true).await?;
            let workflow_names: Vec<String> = workflows
                .iter()
                .map(|workflow| workflow.display_name.clone().unwrap_or_else(|| workflow.name.clone()))
                .collect();

            let track_search = tokio::task::spawn(async move {
                let a: [(&'static str, &'static str); 0] = []; // dumb
                fig_telemetry::emit_track(TrackEvent::Other("Workflow Searched".into()), TrackSource::Cli, a)
                    .await
                    .ok();
            });

            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    let selection = {
                        use std::io::Cursor;

                        use skim::prelude::*;

                        let input = workflow_names.join("\n");
                        let item_reader = SkimItemReader::default();
                        let items = item_reader.of_bufread(Cursor::new(input));
                        let output = Skim::run_with(
                            &SkimOptionsBuilder::default().height(Some("50%")).build().unwrap(),
                            Some(items),
                        ).unwrap();

                        if output.is_abort {
                            return Ok(());
                        }

                        match output.selected_items.get(0).and_then(|selected| {
                            let name = selected.text();
                            workflow_names.iter().enumerate().find(|(_, workflow)| workflow == &&name).map(|(i, _)| i)
                        }) {
                            Some(i) => i,
                            None => return Ok(())
                        }
                    };
                } else if #[cfg(windows)] {
                    let selection = dialoguer::FuzzySelect::with_theme(&crate::util::dialoguer_theme())
                        .items(&workflow_names)
                        .default(0)
                        .interact()
                        .unwrap();
                }
            };

            track_search.await?;
            workflows.remove(selection)
        },
    };

    let track_execution = tokio::task::spawn(async move {
        fig_telemetry::emit_track(TrackEvent::Other("Workflow Executed".into()), TrackSource::Cli, [(
            "execution_method",
            match name {
                Some(_) => "invoke",
                None => "search",
            },
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

    let mut command = format!("fig run {}", workflow.name);
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
