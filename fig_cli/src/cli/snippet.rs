use std::process::Command;

use anyhow::{
    anyhow,
    Result,
};
use reqwest::Method;
use serde::{
    Deserialize,
    Serialize,
};
use tui::components::{
    CollapsiblePicker,
    Frame,
    Label,
    Picker,
    TextField, CheckBox,
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
struct Snippet {
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    parameters: Vec<Parameter>,
}

enum SnippetComponent<'a> {
    CheckBox(CheckBox),
    TextField(TextField),
    Picker(CollapsiblePicker<'a, Picker>),
}

pub async fn execute(name: Option<String>) -> Result<()> {
    let snippets: Vec<Snippet> = request(Method::GET, "/snippets", None, true).await?;

    let snippet = match name {
        Some(name) => match snippets.iter().find(|snippet| snippet.name == name) {
            Some(snippet) => snippet,
            None => return Err(anyhow!("No snippet with name: {}", name)),
        },
        None => {
            let snippet_names: Vec<&str> = snippets.iter().map(|snippet| snippet.name.as_ref()).collect();
            let selection = dialoguer::FuzzySelect::with_theme(&crate::util::dialoguer_theme())
                .items(&snippet_names)
                .default(0)
                .interact()
                .unwrap();
            &snippets[selection]
        },
    };

    let mut components: Vec<SnippetComponent> = snippet
        .parameters
        .iter()
        .map(|param| match &param.parameter_type {
            ParameterType::Checkbox {
                true_value_substitution,
                false_value_substitution,
            } => SnippetComponent::CheckBox(CheckBox::new(false)),
            ParameterType::Text { placeholder } => match placeholder {
                Some(hint) => SnippetComponent::TextField(TextField::new().with_hint(hint)),
                None => SnippetComponent::TextField(TextField::new()),
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
                                    for option in String::from_utf8_lossy(&output.stdout).split("\n") {
                                        if !option.is_empty() {
                                            options.push(option.to_owned());
                                        }
                                    }
                                }
                            },
                        }
                    }
                }

                SnippetComponent::Picker(CollapsiblePicker::new(options))
            },
        })
        .collect();

    let mut frames: Vec<Frame> = components
        .iter_mut()
        .enumerate()
        .map(|(i, component)| {
            let component = match component {
                SnippetComponent::CheckBox(c) => c as &mut dyn Component,
                SnippetComponent::TextField(c) => c as &mut dyn Component,
                SnippetComponent::Picker(c) => c as &mut dyn Component,
            };

            Frame::new(component).with_title(snippet.parameters[i].display_name.as_ref().unwrap_or(&snippet.parameters[i].name))
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
    let mut description = snippet.description.as_ref().map(|description| Label::new(description).with_margin_bottom(1));
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
        .run::<std::io::Error, _>(
            ControlFlow::Wait,
            DisplayMode::AlternateScreen,
            &mut Form::new(model),
        )?;

    Ok(())
}
