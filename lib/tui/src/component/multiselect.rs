use termwiz::cell::unicode_column_width;
use termwiz::color::ColorAttribute;
use termwiz::surface::Surface;

use super::shared::ListState;
use super::ComponentData;
use crate::event_loop::State;
use crate::input::{
    InputAction,
    MouseAction,
};
use crate::surface_ext::SurfaceExt;
use crate::{
    Component,
    Event,
};

#[derive(Debug)]
pub enum MultiselectEvent {
    /// The user has selected an option
    OptionsSelected { id: String, options: Vec<String> },
}

#[derive(Debug)]
pub struct Multiselect {
    list_state: ListState,
    hint: Option<String>,
    selection: Vec<usize>,
    inner: ComponentData,
}

impl Multiselect {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            list_state: ListState::new(options),
            hint: None,
            selection: vec![],
            inner: ComponentData::new("select".to_owned(), true),
        }
    }

    pub fn set_options(&mut self, options: Vec<String>) {
        self.list_state = ListState::new(options);
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = id.into();
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.inner.classes.push(class.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl Component for Multiselect {
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }

        let style = self.style(state);

        let sorted_options = self.list_state.sorted_options();
        let arrow = match self.inner.focus & !sorted_options.is_empty() {
            true => '▿',
            false => '▹',
        };

        surface.draw_text(arrow, x, y, 1.0, style.attributes());

        match self.selection.is_empty() {
            true => {
                let mut attributes = style.attributes();
                attributes.set_foreground(ColorAttribute::PaletteIndex(8));

                if let Some(hint) = &self.hint {
                    surface.draw_text(hint, x + 2.0, y, width - 2.0, attributes);
                }
            },
            false => {
                let mut options = vec![];
                for selection in &self.selection {
                    options.push(self.list_state.options()[*selection].to_owned());
                }

                let text = options.join(", ");
                surface.draw_text(text, x + 2.0, y, width - 2.0, style.attributes());
            },
        }

        for (i, option) in sorted_options.iter().enumerate() {
            if i + 1 >= height as usize {
                return;
            }

            let mut attributes = style.attributes();
            attributes.set_foreground(ColorAttribute::PaletteIndex(2));

            surface.draw_text(
                match self.selection.contains(&(i + self.list_state.index_offset())) {
                    true => "✔ ",
                    false => "  ",
                },
                x,
                y + i as f64 + 1.0,
                2.0,
                attributes,
            );

            let mut attributes = style.attributes();
            if let Some(index) = self.list_state.visible_index() {
                if i == index {
                    attributes
                        .set_background(style.color())
                        .set_foreground(ColorAttribute::PaletteIndex(0));
                }
            }

            let width = width - 2.0;
            let text_width = unicode_column_width(option, None) as f64;
            match width < text_width {
                true => surface.draw_text(
                    format!("{}...", option[..(width - 3.0).max(0.0) as usize].trim_end()),
                    x + 2.0,
                    y + i as f64 + 1.0,
                    width,
                    attributes,
                ),
                false => surface.draw_text(option, x + 2.0, y + i as f64 + 1.0, width, attributes),
            }
        }
    }

    fn on_input_action(&mut self, _: &mut State, input_action: &InputAction) {
        match input_action {
            InputAction::Up => self.list_state.prev(),
            InputAction::Down => self.list_state.next(),
            InputAction::Insert(' ') => {
                if let Some(index) = self.list_state.index() {
                    match self.selection.iter().position(|selection| *selection == index) {
                        Some(position) => {
                            self.selection.remove(position);
                        },
                        None => self.selection.push(index),
                    }
                }
            },
            _ => (),
        }
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        match focus {
            true => self.list_state.sort(""),
            false => {
                let mut options = vec![];
                for selection in &self.selection {
                    options.push(self.list_state.options()[*selection].to_owned());
                }

                state
                    .event_buffer
                    .push(Event::Multiselect(MultiselectEvent::OptionsSelected {
                        id: self.inner.id.to_owned(),
                        options,
                    }));
            },
        }
    }

    fn on_mouse_action(&mut self, _: &mut State, mouse_action: &MouseAction, _: f64, y: f64, _: f64, _: f64) {
        if self.inner.focus {
            let index = (mouse_action.y - y).round() as usize;

            if index > 0 {
                self.list_state.set_index(index.saturating_sub(1));
                if mouse_action.just_pressed {
                    if let Some(index) = self.list_state.visible_index() {
                        match self
                            .selection
                            .iter()
                            .position(|selection| selection.saturating_add(self.list_state.index_offset()) == index)
                        {
                            Some(position) => {
                                self.selection.remove(position);
                            },
                            None => self.selection.push(index),
                        }
                    }
                }
            }
        }
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        let mut w = 60.max(
            self.list_state
                .options()
                .iter()
                .fold(0, |acc, option| acc.max(unicode_column_width(option, None))),
        );

        if let Some(hint) = &self.hint {
            w = w.max(unicode_column_width(hint, None));
        }

        let height = match self.inner.focus & !self.list_state.sorted_options().is_empty() {
            true => 1.0 + self.list_state.options().len().min(self.list_state.max_rows()) as f64,
            false => 1.0,
        };

        (w as f64, height)
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}
