use termwiz::color::ColorAttribute;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Surface,
};
use unicode_width::UnicodeWidthStr;

use crate::component::text_state::TextState;
use crate::component::ComponentData;
use crate::event_loop::{
    Event,
    State,
};
use crate::input::{
    InputAction,
    MouseAction,
};
use crate::surface_ext::SurfaceExt;
use crate::Component;

#[derive(Debug)]
pub enum TextFieldEvent {
    TextChanged { id: Option<String>, text: String },
}

#[derive(Debug)]
pub struct TextField {
    text: TextState,
    hint: Option<String>,
    obfuscated: bool,
    inner: ComponentData,
}

impl TextField {
    pub fn new() -> Self {
        Self {
            text: TextState::new(""),
            hint: None,
            obfuscated: false,
            inner: ComponentData::new("input".to_owned(), true),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = Some(id.into());
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.inner.classes.push(class.into());
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = TextState::new(text.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// This function replaces all characters of the visible text with the '*' character
    pub fn obfuscated(mut self, obfuscated: bool) -> Self {
        self.obfuscated = obfuscated;
        self
    }
}

impl Component for TextField {
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let style = self.style(state);

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
                    false => surface.draw_text(
                        &self.text.as_str()[self.text.cursor.saturating_sub((width.round() - 1.0) as usize)..],
                        x,
                        y,
                        width,
                        style.attributes(),
                    ),
                };
            },
        };

        if self.inner.focus {
            state.cursor_position = (x + (self.text.cursor as f64).min(width.round() - 1.0), y);
            state.cursor_color = style.caret_color();
            surface.add_change(Change::CursorVisibility(CursorVisibility::Visible));
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {
        if self.text.on_input_action(input_action).is_err() {
            return;
        }

        if !self.text.is_empty() {
            state.event_buffer.push(Event::TextField(TextFieldEvent::TextChanged {
                id: self.inner.id.to_owned(),
                text: self.text.to_owned(),
            }))
        }
    }

    fn on_mouse_action(&mut self, state: &mut State, mouse_action: &MouseAction, x: f64, _: f64, _: f64, _: f64) {
        self.text.on_mouse_action(mouse_action, x);

        if !self.text.is_empty() {
            state.event_buffer.push(Event::TextField(TextFieldEvent::TextChanged {
                id: self.inner.id.to_owned(),
                text: self.text.to_owned(),
            }))
        }
    }

    fn inner(&self) -> &ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        (
            match self.text.is_empty() {
                true => match &self.hint {
                    Some(hint) => hint.width() as f64,
                    None => 0.0,
                },
                false => self.text.width() as f64,
            }
            .max(80.0),
            1.0,
        )
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}

impl Default for TextField {
    fn default() -> Self {
        Self::new()
    }
}
