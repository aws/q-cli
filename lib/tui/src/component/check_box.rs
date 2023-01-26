use std::fmt::Display;

use termwiz::cell::unicode_column_width;
use termwiz::surface::Surface;

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
pub enum CheckBoxEvent {
    /// The user has either checked or unchecked the box
    Checked { id: Option<String>, checked: bool },
}

#[derive(Debug)]
pub struct CheckBox {
    label: String,
    checked: bool,
    inner: ComponentData,
}

impl CheckBox {
    pub fn new(label: impl Display, checked: bool) -> Self {
        Self {
            label: label.to_string(),
            checked,
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
}

impl Component for CheckBox {
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, _: f64, _: f64) {
        let style = self.style(state);

        surface.draw_text(
            &format!("{} {}", if self.checked { '☑' } else { '☐' }, self.label),
            x,
            y,
            2.0 + unicode_column_width(&self.label, None) as f64,
            style.attributes(),
        );
    }

    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {
        if let InputAction::Insert(' ') = input_action {
            self.checked = !self.checked;
            state.event_buffer.push(Event::CheckBox(CheckBoxEvent::Checked {
                id: self.inner.id.to_owned(),
                checked: self.checked,
            }))
        }
    }

    fn on_mouse_action(&mut self, _: &mut State, mouse_action: &MouseAction, _: f64, _: f64, _: f64, _: f64) {
        if mouse_action.just_pressed && self.inner.focus {
            self.checked = !self.checked;
        }
    }

    fn inner(&self) -> &ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        (2.0 + unicode_column_width(&self.label, None) as f64, 1.0)
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}
