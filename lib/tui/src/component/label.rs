use std::fmt::Display;

use termwiz::surface::Surface;

use super::ComponentData;
use crate::event_loop::State;
use crate::surface_ext::SurfaceExt;
use crate::Component;

#[derive(Debug)]
pub struct Label {
    label: String,
    bold: bool,
    inner: ComponentData,
}

impl Label {
    pub fn new(id: impl ToString, label: impl Display, bold: bool) -> Self {
        Self {
            label: label.to_string(),
            bold,
            inner: ComponentData::new(id.to_string(), false),
        }
    }
}

impl Component for Label {
    fn initialize(&mut self, _: &mut State) {
        self.inner.width = self.label.len() as f64;
        self.inner.height = 1.0;
    }

    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, _: f64, _: f64) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let style = self.style(state);

        surface.draw_text(
            &self.label[0..self.label.len().min(width as usize)],
            x,
            y,
            style.color(),
            style.background_color(),
            self.bold,
        );
    }

    fn class(&self) -> &'static str {
        "p"
    }

    fn inner(&self) -> &ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut ComponentData {
        &mut self.inner
    }
}
