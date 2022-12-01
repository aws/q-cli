use termwiz::color::ColorAttribute;
use termwiz::surface::Surface;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::component::ComponentData;
use crate::event_loop::{
    Event,
    State,
};
use crate::input::InputAction;
use crate::surface_ext::SurfaceExt;
use crate::Component;

#[derive(Debug)]
pub enum TextFieldEvent {
    TextChanged { id: String, text: String },
}

#[derive(Debug)]
pub struct TextField {
    text: String,
    cursor: usize,
    offset: usize,
    hint: Option<String>,
    obfuscated: bool,
    inner: ComponentData,
}

impl TextField {
    pub fn new(id: impl ToString) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            offset: 0,
            hint: None,
            obfuscated: false,
            inner: ComponentData::new(id.to_string(), true),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor = self.text.width();
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
}

impl Component for TextField {
    fn initialize(&mut self, _: &mut State) {
        self.inner.width = 32.0;
        self.inner.height = 1.0;
    }

    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, _: f64, _: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let style = self.style(state);
        let width = style.width().unwrap_or(width);

        match self.text.is_empty() {
            true => {
                if let Some(hint) = &self.hint {
                    let mut attributes = style.attributes();
                    attributes.set_foreground(ColorAttribute::PaletteIndex(8));
                    surface.draw_text(hint.as_str(), x, y, width, attributes);
                }

                if self.inner.focus {
                    let mut attributes = style.attributes();
                    attributes
                        .set_foreground(style.background_color())
                        .set_background(style.caret_color());

                    surface.draw_text(
                        self.hint
                            .as_ref()
                            .and_then(|hint| hint.graphemes(true).nth(self.cursor))
                            .unwrap_or(" "),
                        x + self.cursor as f64 - self.offset as f64,
                        y,
                        1.0,
                        attributes,
                    );
                }
            },
            false => {
                match self.obfuscated {
                    true => surface.draw_text("*".repeat(self.text.len()), x, y, width, style.attributes()),
                    false => surface.draw_text(&self.text.as_str()[self.offset..], x, y, width, style.attributes()),
                };

                if self.inner.focus {
                    let mut attributes = style.attributes();
                    attributes
                        .set_foreground(style.background_color())
                        .set_background(style.caret_color());

                    surface.draw_text(
                        self.text.graphemes(true).nth(self.cursor).unwrap_or(" "),
                        x + self.cursor as f64 - self.offset as f64,
                        y,
                        1.0,
                        attributes,
                    );
                }
            },
        };
    }

    fn on_input_action(&mut self, state: &mut State, input_action: InputAction) -> bool {
        match input_action {
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
                        return true;
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
            state.event_buffer.push(Event::TextField(TextFieldEvent::TextChanged {
                id: self.inner.id.to_owned(),
                text: self.text.to_owned(),
            }))
        }

        true
    }

    fn on_resize(&mut self, _: &mut State, width: f64, _: f64) {
        let width = width.round() as usize;

        if self.cursor >= width {
            self.offset = self.cursor - width;
        }
    }

    fn class(&self) -> &'static str {
        "input:text"
    }

    fn inner(&self) -> &ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut ComponentData {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use termwiz::input::{
        InputEvent,
        KeyCode,
        KeyEvent,
        Modifiers,
    };

    use super::*;
    use crate::{
        ControlFlow,
        EventLoop,
        InputMethod,
        StyleSheet,
    };

    #[ignore = "does not work on CI"]
    #[test]
    fn test_text_field() {
        let mut test = String::new();

        let text_field_id = "test";
        let mut text_field = TextField::new("test");

        EventLoop::new()
            .run(
                &mut text_field,
                InputMethod::Scripted(vec![InputEvent::Key(KeyEvent {
                    key: KeyCode::Char('a'),
                    modifiers: Modifiers::NONE,
                })]),
                StyleSheet::default(),
                |event, _component, control_flow| {
                    if let Event::TextField(TextFieldEvent::TextChanged { id, text }) = event {
                        if id == text_field_id {
                            test = text;
                            *control_flow = ControlFlow::Quit
                        }
                    }
                },
            )
            .unwrap();

        assert_eq!(test, "a");
    }
}
