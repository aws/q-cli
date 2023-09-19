use crossterm::style::Stylize;
#[cfg(unix)]
use skim::SkimItem;

#[cfg(unix)]
pub enum ScriptAction {
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
