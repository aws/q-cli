use termwiz::color::ColorAttribute;
use termwiz::surface::Surface;
use unicode_width::UnicodeWidthStr;

use crate::component::text_state::TextState;
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
    text: TextState,
    offset: usize,
    hint: Option<String>,
    obfuscated: bool,
    inner: ComponentData,
}

impl TextField {
    pub fn new(id: impl ToString) -> Self {
        Self {
            text: TextState::new(""),
            offset: 0,
            hint: None,
            obfuscated: false,
            inner: ComponentData::new("input".to_owned(), id.to_string(), true),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = TextState::new(text.into());
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

    fn resize(&mut self) {
        self.inner.width = match self.text.is_empty() {
            true => match &self.hint {
                Some(hint) => hint.width() as f64,
                None => 0.0,
            },
            false => self.text.width() as f64,
        };

        self.inner.height = 1.0;
    }
}

impl Component for TextField {
    fn initialize(&mut self, _: &mut State) {
        self.resize();
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
            },
            false => {
                match self.obfuscated {
                    true => surface.draw_text("*".repeat(self.text.len()), x, y, width, style.attributes()),
                    false => surface.draw_text(&self.text.as_str()[self.offset..], x, y, width, style.attributes()),
                };
            },
        };

        if self.inner.focus {
            state.cursor_position = (x + self.text.cursor as f64 - self.offset as f64, y);
            state.cursor_color = style.caret_color();
            state.cursor_visibility = true;
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: InputAction) -> Option<bool> {
        if self.text.on_input_action(&input_action).is_err() {
            return None;
        }

        if !self.text.is_empty() {
            state.event_buffer.push(Event::TextField(TextFieldEvent::TextChanged {
                id: self.inner.id.to_owned(),
                text: self.text.to_owned(),
            }))
        }

        self.resize();
        None
    }

    fn on_paste(&mut self, _: &mut State, clipboard: &str) {
        self.text.insert_str(clipboard);
        self.resize();
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
    use lightningcss::stylesheet::{
        ParserOptions,
        StyleSheet,
    };
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
                StyleSheet::parse("", ParserOptions::default()).unwrap(),
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
