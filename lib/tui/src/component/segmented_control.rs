use termwiz::cell::unicode_column_width;
use termwiz::color::ColorAttribute;
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
pub enum SegmentedControlEvent {
    /// The user has either checked or unchecked the box
    SelectionChanged { id: String, selection: String },
}

#[derive(Debug)]
pub struct SegmentedControl {
    index: usize,
    options: Vec<String>,
    inner: ComponentData,
}

impl SegmentedControl {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            index: 0,
            options,
            inner: ComponentData::new("input".to_owned(), true),
        }
    }

    pub fn with_index(mut self, index: usize) -> Self {
        self.index = index;
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = id.into();
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.inner.classes.push(class.into());
        self
    }

    fn push_selection_changed_event(&self, state: &mut State) {
        state
            .event_buffer
            .push(Event::SegmentedControl(SegmentedControlEvent::SelectionChanged {
                id: self.inner.id.to_owned(),
                selection: self.options[self.index].to_owned(),
            }))
    }
}

impl Component for SegmentedControl {
    fn draw(&self, state: &mut State, surface: &mut Surface, mut x: f64, y: f64, _: f64, _: f64) {
        let style = self.style(state);

        for (i, option) in self.options.iter().enumerate() {
            let mut attributes = style.attributes();

            if i == self.index {
                attributes
                    .set_background(style.color())
                    .set_foreground(ColorAttribute::PaletteIndex(0));
            }

            surface.draw_text(
                format!(" {option} "),
                x,
                y,
                unicode_column_width(option, None) as f64 + 2.0,
                attributes,
            );

            x += unicode_column_width(option, None) as f64 + 2.0;

            if i < self.options.len().saturating_sub(1) {
                surface.draw_text(" ", x, y, 1.0, style.attributes());
                x += 1.0;
            }
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {
        match input_action {
            InputAction::Left => {
                match self.index == 0 {
                    true => self.index = self.options.len().saturating_sub(1),
                    false => self.index = self.index.saturating_sub(1),
                };
                self.push_selection_changed_event(state);
            },
            InputAction::Right => {
                if let Some(index) = self.index.saturating_add(1).checked_rem(self.options.len()) {
                    self.index = index;
                    self.push_selection_changed_event(state);
                }
            },
            _ => (),
        }
    }

    fn on_mouse_action(&mut self, state: &mut State, mouse_action: &MouseAction, x: f64, _: f64, _: f64, _: f64) {
        if mouse_action.just_pressed && self.inner.focus {
            let mouse_x = mouse_action.x - x;
            let mut acc = 0;
            for (i, option) in self.options.iter().enumerate() {
                if mouse_x < (acc + unicode_column_width(option, None).saturating_add(3)) as f64 {
                    self.index = i;
                    self.push_selection_changed_event(state);
                    break;
                }

                acc += unicode_column_width(option, None).saturating_add(3);
            }
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
            (self
                .options
                .iter()
                .fold(0, |acc, option| acc + unicode_column_width(option, None) + 2)
                + self.options.len().saturating_sub(1)) as f64,
            1.0,
        )
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}
