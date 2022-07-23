use std::fmt::Display;

use newton::DisplayState;

use crate::Style;

#[derive(Debug, Clone, Default)]
pub struct Label {
    pub label: String,
}

impl Label {
    pub fn new(label: impl Display) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub(crate) fn initialize(&mut self, width: &mut i32, height: &mut i32) {
        *width = i32::try_from(self.label.len()).unwrap();
        *height = 1;
    }

    pub(crate) fn draw(&self, renderer: &mut DisplayState, style: &Style, x: i32, y: i32, width: i32, height: i32) {
        if height <= 0 {
            return;
        }

        if let Ok(width) = usize::try_from(width) {
            renderer.draw_string(
                &self.label[0..self.label.len().min(width)],
                x,
                y,
                style.color(),
                style.background_color(),
            );
        }
    }
}
