use newton::{
    Color,
    DisplayState,
};

use crate::input::InputAction;
use crate::Style;

pub struct TextField {
    text: String,
    cursor: usize,
    offset: usize,
    hint: Option<String>,
    obfuscated: bool,
    focused: bool,
    signal: Box<dyn Fn(String)>,
}

impl TextField {
    pub fn new(signal: impl Fn(String) + 'static) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            offset: 0,
            hint: None,
            obfuscated: false,
            focused: false,
            signal: Box::new(signal),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor = self.text.len();
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn obfuscated(mut self, obfuscated: bool) -> Self {
        self.obfuscated = obfuscated;
        self
    }

    pub(crate) fn initialize(&mut self, width: &mut i32, height: &mut i32) {
        *width = 32;
        *height = 1;
    }

    pub(crate) fn draw(&self, renderer: &mut DisplayState, style: &Style, x: i32, y: i32, width: i32, height: i32) {
        if height <= 0 || width <= 0 {
            return;
        }

        let width = match usize::try_from(width) {
            Ok(width) => width,
            _ => return,
        };

        match self.text.is_empty() {
            true => match &self.hint {
                Some(hint) => renderer.draw_string(
                    &hint.as_str()[self.offset..hint.len().min(width + self.offset)],
                    x,
                    y,
                    Color::DarkGrey,
                    style.background_color(),
                ),
                None => renderer,
            },
            false => {
                match self.obfuscated {
                    true => renderer.draw_string(
                        "*".repeat(self.text.len().min(width)),
                        x,
                        y,
                        style.color(),
                        style.background_color(),
                    ),
                    false => renderer.draw_string(
                        &self.text.as_str()[self.offset..self.text.len().min(width + self.offset)],
                        x,
                        y,
                        style.color(),
                        style.background_color(),
                    ),
                };

                if self.focused {
                    renderer.draw_symbol(
                        self.text.chars().nth(self.cursor).unwrap_or(' '),
                        x + i32::try_from(self.cursor).unwrap() - i32::try_from(self.offset).unwrap(),
                        y,
                        style.background_color(),
                        style.color(),
                    );
                }

                renderer
            },
        };
    }

    pub(crate) fn on_input_action(&mut self, input: InputAction) {
        match input {
            InputAction::Left => self.cursor -= 1.min(self.cursor),
            InputAction::Right => self.cursor += 1.min(self.text.len() - self.cursor),
            InputAction::Insert(c, _) => {
                self.text.insert(self.cursor, c);
                self.cursor += 1;
            },
            InputAction::Remove => match self.cursor == self.text.len() {
                true => {
                    self.text.pop();
                    self.cursor -= 1.min(self.cursor);
                },
                false => {
                    if self.cursor == 0 {
                        return;
                    }

                    self.text.remove(self.cursor - 1);
                    self.cursor -= 1.min(self.cursor);
                },
            },
            InputAction::Delete => match self.text.len() {
                len if len == self.cursor + 1 => {
                    self.text.pop();
                },
                len if len > self.cursor + 1 => {
                    self.text.remove(self.cursor);
                },
                _ => (),
            },
            _ => (),
        }

        if !self.text.is_empty() {
            (self.signal)(self.text.clone());
        }
    }

    pub(crate) fn on_focus(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub(crate) fn on_resize(&mut self, width: i32) {
        if let Ok(width) = usize::try_from(width) {
            if self.cursor >= width {
                self.offset = self.cursor - width;
            }
        }
    }
}
