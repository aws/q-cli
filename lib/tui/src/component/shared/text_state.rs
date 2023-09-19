use std::fmt::Display;

use termwiz::cell::unicode_column_width;
use termwiz::input::MouseButtons;
use unicode_segmentation::UnicodeSegmentation;

use crate::input::MouseAction;

#[derive(Debug, Default)]
pub struct TextState {
    text: String,
    byte_index: usize,
    grapheme_index: usize,
}

impl TextState {
    pub fn new(s: impl Into<String>) -> Self {
        let text: String = s.into();
        let byte_index = text.len();
        let grapheme_index = unicode_column_width(&text, None);

        Self {
            text,
            byte_index,
            grapheme_index,
        }
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.byte_index = self.text.len();
        self.grapheme_index = unicode_column_width(&self.text, None);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn grapheme_index(&self) -> usize {
        self.grapheme_index
    }

    pub fn left(&mut self) {
        if self.grapheme_index == 0 {
            return;
        }

        if let Some(grapheme) = self.text.graphemes(true).nth(self.grapheme_index - 1) {
            self.byte_index = self.byte_index.saturating_sub(grapheme.len());
            self.grapheme_index = self.grapheme_index.saturating_sub(1);
        }
    }

    pub fn right(&mut self) {
        if self.grapheme_index == unicode_column_width(&self.text, None) {
            return;
        }

        if let Some(grapheme) = self.text.graphemes(true).nth(self.grapheme_index) {
            self.byte_index = self.byte_index.saturating_add(grapheme.len());
            self.grapheme_index = self.grapheme_index.saturating_add(1);
        }
    }

    pub fn character(&mut self, character: char) {
        self.text.insert(self.byte_index, character);
        self.right();
    }

    pub fn backspace(&mut self) {
        if self.grapheme_index == 0 {
            return;
        }

        self.left();
        self.delete();
    }

    pub fn delete(&mut self) {
        if self.grapheme_index >= unicode_column_width(&self.text, None) {
            return;
        }

        if let Some(grapheme) = self.text.graphemes(true).nth(self.grapheme_index) {
            self.text
                .replace_range(self.byte_index..self.byte_index + grapheme.len(), "");
        }
    }

    pub fn paste(&mut self, clipboard: &str) {
        self.text.insert_str(self.byte_index, clipboard);
        self.byte_index = self.byte_index.saturating_add(clipboard.len());
        self.grapheme_index = self
            .grapheme_index
            .saturating_add(unicode_column_width(clipboard, None));
    }

    pub fn on_mouse_action(&mut self, mouse_action: &MouseAction, x: f64) {
        if mouse_action.buttons.contains(MouseButtons::LEFT) && mouse_action.x >= x {
            self.grapheme_index = ((mouse_action.x - x) as usize).min(unicode_column_width(&self.text, None));
            self.byte_index = self.text.graphemes(true).collect::<Vec<&str>>()[0..self.grapheme_index]
                .iter()
                .fold(0, |acc, grapheme| acc + grapheme.len());
        }
    }
}

impl Display for TextState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}
