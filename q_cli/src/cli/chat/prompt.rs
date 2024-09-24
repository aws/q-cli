use std::borrow::Cow;

use crossterm::style::Stylize;
use eyre::Result;
use rustyline::completion::{
    Completer,
    extract_word,
};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::history::DefaultHistory;
use rustyline::{
    Completer,
    CompletionType,
    Config,
    Context,
    EditMode,
    Editor,
    Helper,
    Hinter,
    Validator,
};
use winnow::stream::AsChar;

pub const PROMPT: &str = "> ";

const MODIFIERS: &[&str] = &["@history", "@git", "@env"];

pub struct ChatCompleater {}

impl ChatCompleater {
    fn new() -> Self {
        Self {}
    }
}

impl Completer for ChatCompleater {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        let (start, word) = extract_word(line, pos, None, |c| c.is_space());
        Ok((
            start,
            if word.starts_with('@') {
                MODIFIERS
                    .iter()
                    .filter(|p| p.starts_with(word))
                    .map(|s| (*s).to_owned())
                    .collect()
            } else {
                Vec::new()
            },
        ))
    }
}

#[derive(Helper, Completer, Hinter, Validator)]
pub struct ChatHelper {
    #[rustyline(Completer)]
    completer: ChatCompleater,
    #[rustyline(Validator)]
    validator: (),
    #[rustyline(Hinter)]
    hinter: (),
}

impl Highlighter for ChatHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, default: bool) -> Cow<'b, str> {
        if default {
            Cow::Owned(PROMPT.magenta().to_string())
        } else {
            Cow::Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[1m{hint}\x1b[m"))
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        false
    }
}

pub fn rl() -> Result<Editor<ChatHelper, DefaultHistory>> {
    let edit_mode = match fig_settings::settings::get_string_opt("chat.editMode").as_deref() {
        Some("vi" | "vim") => EditMode::Vi,
        _ => EditMode::Emacs,
    };
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(edit_mode)
        .build();
    let h = ChatHelper {
        completer: ChatCompleater::new(),
        hinter: (),
        validator: (),
    };
    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(h));
    Ok(rl)
}
