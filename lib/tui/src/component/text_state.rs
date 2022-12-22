use std::ops::{
    Deref,
    DerefMut,
};

use termwiz::input::MouseButtons;
use unicode_width::UnicodeWidthStr;

use crate::input::InputAction;

#[derive(Debug, Default)]
pub struct TextState {
    pub text: String,
    pub cursor: usize,
}

impl TextState {
    pub fn new(s: impl Into<String>) -> Self {
        let text = s.into();
        let cursor = text.width();
        Self { text, cursor }
    }

    pub fn on_input_action(&mut self, input_action: &InputAction) -> Result<(), &'static str> {
        let cursor = self.cursor;
        match input_action {
            InputAction::Left => self.cursor -= 1.min(cursor),
            InputAction::Right => self.cursor += 1.min(self.width() - cursor),
            InputAction::Insert(c) => {
                self.insert(cursor, *c);
                self.cursor += 1;
            },
            InputAction::Remove => match cursor == self.len() {
                true => {
                    self.pop();
                    self.cursor -= 1.min(cursor);
                },
                false => {
                    if cursor == 0 {
                        return Err("Tried to remove string with cursor at index 0.");
                    }

                    self.remove(cursor - 1);
                    self.cursor -= 1.min(self.cursor);
                },
            },
            InputAction::Delete => match self.len() {
                len if len == cursor + 1 => {
                    self.pop();
                },
                len if len > cursor + 1 => {
                    self.remove(cursor);
                },
                _ => (),
            },
            InputAction::Paste(clipboard) => {
                self.text.insert_str(self.cursor, clipboard);
                self.cursor += clipboard.width();
            },
            _ => (),
        }

        Ok(())
    }

    pub fn on_mouse_event(&mut self, mouse_event: &termwiz::input::MouseEvent, x: f64) {
        if mouse_event.mouse_buttons.contains(MouseButtons::LEFT) {
            self.cursor = ((f64::from(mouse_event.x) - x).max(0.0) as usize).min(self.text.len());
        }
    }
}

impl Deref for TextState {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl DerefMut for TextState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text
    }
}
