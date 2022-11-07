use std::fmt::Display;

use newton::DisplayState;

use crate::input::InputAction;
use crate::Style;

pub struct CheckBox {
    label: String,
    checked: bool,
    signal: Box<dyn Fn(bool)>,
}

impl CheckBox {
    pub fn new(label: impl Display, checked: bool, signal: impl Fn(bool) + 'static) -> Self {
        Self {
            label: label.to_string(),
            checked,
            signal: Box::new(signal),
        }
    }

    pub(crate) fn initialize(&mut self, width: &mut i32, height: &mut i32) {
        *width = 4 + i32::try_from(self.label.len()).unwrap();
        *height = 1;
    }

    pub(crate) fn draw(&self, renderer: &mut DisplayState, style: &Style, x: i32, y: i32, width: i32, height: i32) {
        if height <= 0 {
            return;
        }

        if let Ok(width) = usize::try_from(width) {
            renderer.draw_string(
                &format!("{} {}", if self.checked { '☑' } else { '☐' }, self.label)
                    [0..(4 + self.label.len()).min(width)],
                x,
                y,
                style.color(),
                style.background_color(),
                false,
            );
        }
    }

    pub(crate) fn on_input_action(&mut self, input: InputAction) {
        if let InputAction::Select = input {
            self.checked = !self.checked;
            (self.signal)(self.checked)
        }
    }
}
