use termwiz::surface::Surface;

use super::ComponentData;
use crate::surface_ext::SurfaceExt;
use crate::{
    Component,
    State,
};

#[derive(Debug)]
pub struct Hr {
    inner: ComponentData,
}

impl Hr {
    pub fn new() -> Self {
        Self {
            inner: ComponentData::new("hr".to_owned(), false),
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

impl Component for Hr {
    fn draw(&self, state: &mut crate::event_loop::State, surface: &mut Surface, x: f64, y: f64, width: f64, _: f64) {
        let style = self.style(state);

        surface.draw_text("─".repeat(width.round() as usize), x, y, width, style.attributes());
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        (30.0, 1.0)
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}

impl Default for Hr {
    fn default() -> Self {
        Self::new()
    }
}
