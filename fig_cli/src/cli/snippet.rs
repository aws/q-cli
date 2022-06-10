use std::collections::HashMap;
use std::process::Command;

use anyhow::{
    anyhow,
    Result,
};
use dialoguer::Confirm;
use reqwest::Method;
use serde::{
    Deserialize,
    Serialize,
};
use tui::components::{
    CheckBox,
    CollapsiblePicker,
    Frame,
    Label,
    Picker,
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
    Style,
    StyleSheet,
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
    Token {
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snippet {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    template: String,
    parameters: Vec<Parameter>,
    tree: Vec<TreeElement>,
}

enum SnippetComponent<'a> {
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
        inner: CollapsiblePicker<'a, Picker>,
    },
}

impl<'a> From<Parameter> for SnippetComponent<'a> {
    fn from(from: Parameter) -> Self {
        let display_name = from.display_name.unwrap_or_else(|| from.name.clone());
        let name = from.name;

        match from.parameter_type {
            ParameterType::Checkbox {
                true_value_substitution,
                false_value_substitution,
            } => Self::CheckBox {
                name,
                display_name,
                inner: CheckBox::new(false),
                value_if_true: true_value_substitution,
                value_if_false: false_value_substitution,
            },
            ParameterType::Text { placeholder } => Self::TextField {
                name,
                display_name,
                inner: match placeholder {
                    Some(hint) => TextField::new().with_hint(hint),
                    None => TextField::new(),
                },
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
                            Generator::Named { name } => todo!(),
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

                Self::Picker {
                    name,
                    display_name,
                    inner: CollapsiblePicker::new(options),
                }
            },
        }
    }
}

pub async fn execute(name: Option<String>, args: Option<HashMap<String, String>>) -> Result<()> {
    let snippet = match name {
        Some(name) => match request(Method::GET, format!("/snippets/{name}"), None, true).await? {
            Some(snippet) => snippet,
            None => return Err(anyhow!("Snippet does not exist with name: {}", name)),
        },
        None => {
            let mut snippets: Vec<Snippet> = request(Method::GET, "/snippets", None, true).await?;
            let snippet_names: Vec<&str> = snippets.iter().map(|snippet| snippet.name.as_ref()).collect();
            let selection = dialoguer::FuzzySelect::with_theme(&crate::util::dialoguer_theme())
                .items(&snippet_names)
                .default(0)
                .interact()
                .unwrap();
            snippets.remove(selection)
        },
    };

    let tree = snippet.tree;

    let mut components: Vec<SnippetComponent> = snippet.parameters.into_iter().map(SnippetComponent::from).collect();

    let mut frames: Vec<Frame> = components
        .iter_mut()
        .map(|component| match component {
            SnippetComponent::CheckBox {
                display_name, inner, ..
            } => Frame::new(inner as &mut dyn Component).with_title(display_name.to_owned()),
            SnippetComponent::TextField {
                display_name, inner, ..
            } => Frame::new(inner as &mut dyn Component).with_title(display_name.to_owned()),
            SnippetComponent::Picker {
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

    let focus_style = Style::new()
        .with_border_left_color(Color::White)
        .with_border_right_color(Color::White)
        .with_border_top_color(Color::White)
        .with_border_bottom_color(Color::White)
        .with_border_style(thick_border);

    let unfocused_style = Style::new()
        .with_border_left_width(1)
        .with_border_top_width(1)
        .with_border_bottom_width(1)
        .with_border_right_width(1)
        .with_border_left_color(Color::DarkGrey)
        .with_border_right_color(Color::DarkGrey)
        .with_border_top_color(Color::DarkGrey)
        .with_border_bottom_color(Color::DarkGrey)
        .with_border_style(thin_border);

    let style_sheet = StyleSheet::new()
        .with_style(
            "*",
            Style::new()
                .with_border_left_color(Color::Grey)
                .with_border_right_color(Color::Grey)
                .with_border_top_color(Color::Grey)
                .with_border_bottom_color(Color::Grey),
        )
        .with_style("*:focus", focus_style)
        .with_style("frame", unfocused_style)
        .with_style(
            "frame.title",
            Style::new()
                .with_color(Color::DarkGrey)
                .with_padding_left(1)
                .with_padding_right(1)
                .with_margin_left(1),
        )
        .with_style(
            "frame.title:focus",
            Style::new()
                .with_color(Color::White)
                // .with_background_color(Color::White)
                .with_padding_left(1)
                .with_padding_right(1)
                .with_margin_left(1),
        )
        .with_style("frame:focus", focus_style)
        .with_style("textfield", Style::new().with_padding_left(2).with_color(Color::Grey))
        .with_style("textfield:focus", focus_style.with_color(Color::White))
        .with_style("disclosure.summary:focus", Style::new().with_color(Color::White))
        .with_style("disclosure.summary", Style::new().with_color(Color::Cyan))
        .with_style(
            "picker.item",
            Style::new().with_padding_left(2).with_color(Color::DarkGrey),
        )
        .with_style(
            "picker.item:focus",
            Style::new().with_padding_left(2).with_color(Color::White),
        )
        .with_style(
            "picker.selected",
            Style::new()
                .with_margin_left(2)
                .with_background_color(Color::DarkGrey)
                .with_color(Color::Grey),
        )
        .with_style(
            "picker.selected:focus",
            Style::new()
                .with_margin_left(2)
                .with_background_color(Color::White)
                .with_color(Color::DarkGrey),
        );

    let mut model: Vec<&mut dyn Component> = vec![];
    let mut name = Label::new(snippet.display_name.as_ref().unwrap_or(&snippet.name));
    let mut description = snippet
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

    EventLoop::new()
        .with_style_sheet(&style_sheet)
        .run::<std::io::Error, _>(ControlFlow::Wait, DisplayMode::AlternateScreen, &mut Form::new(model))?;

    let mut names: Vec<&str> = vec![];
    let mut args: Vec<&str> = vec![];
    for component in &components {
        match component {
            SnippetComponent::CheckBox {
                name,
                inner,
                value_if_true,
                value_if_false,
                ..
            } => {
                names.push(name);
                args.push(match inner.checked {
                    true => value_if_true,
                    false => value_if_false,
                });
            },
            SnippetComponent::TextField { name, inner, .. } => {
                names.push(name);
                args.push(&inner.text);
            },
            SnippetComponent::Picker { name, inner, .. } => {
                names.push(name);
                args.push(match inner.selected_item() {
                    Some(selected) => selected,
                    None => return Err(anyhow!("Missing entry for field: {name}")),
                });
            },
        }
    }

    let mut command = format!("fig snippet {} ", snippet.name);
    for i in 0..args.len() {
        command.push_str(&format!("{}=\"{}\" ", names[i], args[i]));
    }

    println!("\x1B[1;95mExecuting:\x1B[0m {command}");

    Ok(())
}
